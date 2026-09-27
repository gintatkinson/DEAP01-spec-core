# Task Dispatch: worker_wp12_sync_1 (Git Synchronization Worker)

## Objective
Stage, commit (strictly using neutral citations `refs #...`), push to remote tracking branches across all three repositories (`uav-009`, `uav-011`, and `DEAP01-spec-core`), verify 0-byte remote diff on all three, and update Section 2.1 of `HANDOFF.md` in `DEAP01-spec-core` with verified commit hashes and accurate empirical gate statuses.

## Repositories
1. Customer Workspace: `/Users/perkunas/jail/uav-009` (GitLab remote `origin/main`)
2. Application Workspace: `/Users/perkunas/jail/uav-011` (GitLab remote `origin/main`)
3. Upstream Compiler: `/Users/perkunas/jail/DEAP01-spec-core` (GitHub remote `origin/main`)

## Step 1: Synchronize `uav-009`
1. Check git status in `/Users/perkunas/jail/uav-009`.
2. Stage updated pipeline files (`.pipeline/`, `scripts/`, etc.).
   Confirm zero changes to customer files in `schema/` and `docs/`.
3. Commit with neutral citation:
   ```bash
   git -C /Users/perkunas/jail/uav-009 commit -m "chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate (refs #378, refs #377, refs #376, refs #375, refs #372, refs #366, refs #365, refs #364, refs #362, refs #361, refs #360, refs #349, refs #286)"
   ```
   (If working tree is already committed / clean at HEAD, record current commit hash).
4. Verify commit neutrality:
   `python3 /Users/perkunas/jail/uav-009/scripts/verify_commit_messages.py --head`
5. Push to remote:
   `git -C /Users/perkunas/jail/uav-009 push origin main`
6. Verify remote sync:
   `git -C /Users/perkunas/jail/uav-009 diff origin/main` (must be 0 bytes).
7. Capture HEAD commit hash of `uav-009`.

## Step 2: Synchronize `uav-011`
1. Check git status in `/Users/perkunas/jail/uav-011`.
2. Stage updated pipeline files (`.pipeline/`, `scripts/`, etc.).
   Confirm landing zones remain 100% clean (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` have `.gitkeep` only).
3. Commit with neutral citation:
   ```bash
   git -C /Users/perkunas/jail/uav-011 commit -m "chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate (refs #378, refs #377, refs #376, refs #375, refs #372, refs #366, refs #365, refs #364, refs #362, refs #361, refs #360, refs #349, refs #286)"
   ```
   (If working tree is already committed / clean at HEAD, record current commit hash).
4. Verify commit neutrality:
   `python3 /Users/perkunas/jail/uav-011/scripts/verify_commit_messages.py --head`
5. Push to remote:
   `git -C /Users/perkunas/jail/uav-011 push origin main`
6. Verify remote sync:
   `git -C /Users/perkunas/jail/uav-011 diff origin/main` (must be 0 bytes).
7. Capture HEAD commit hash of `uav-011`.

## Step 3: Update `HANDOFF.md` Section 2.1 in `DEAP01-spec-core`
1. Edit `/Users/perkunas/jail/DEAP01-spec-core/HANDOFF.md` Section 2.1:
   - Update `uav-009` and `uav-011` commit hashes with the verified remote commit hashes from Steps 1 and 2.
   - Update the empirical baseline verification status table: record that all 31 checks (Checks 10 through 31, including Check 31 Dual-Schema SSOT Parity Gate) empirically PASS with exit code 0 when run directly via `python3 scripts/verify_downstream_baseline.py <path>`.
2. Stage `skills/`, `scripts/`, `tests/`, and `HANDOFF.md`.
3. Commit with neutral citation:
   ```bash
   git commit -m "fix(tooling): resolve Check 23 nested AST extraction and Check 30 clean landing zone baseline gating (refs #378, refs #377, refs #376, refs #372)"
   ```
4. Verify commit neutrality:
   `python3 scripts/verify_commit_messages.py --head`
5. Push to GitHub:
   `git push origin main`
6. Verify remote sync:
   `git diff origin/main` (must be 0 bytes).
7. Capture HEAD commit hash of `DEAP01-spec-core`.

## Step 4: Deliver Handoff Report
Write handoff report to `.agents/worker_wp12_sync_1/handoff.md`.
