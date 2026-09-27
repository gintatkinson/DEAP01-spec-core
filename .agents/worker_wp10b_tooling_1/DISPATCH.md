# Task Dispatch: worker_wp10b_tooling_1

## Objective
Refine candidate metric binding and factual grounding logic in `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py` so that Check 23 passes on `uav-009` with exit code 0, without clobbering customer models or specifications, while ensuring all 17 existing Check 23 unit tests pass.

## Inputs
- Upstream Compiler: `/Users/perkunas/jail/DEAP01-spec-core`
- Customer Workspace: `/Users/perkunas/jail/uav-009`
- Application Workspace: `/Users/perkunas/jail/uav-011`
- Target Validator: `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py`
- Test Suite: `tests/test_check23_factual_grounding_gate.py`
- Verification Script: `scripts/verify_downstream_baseline.py`
- Failure Handoff: `.agents/worker_wp11_propagation_1/handoff.md`

## Instructions
1. First step: execute `view_file` on `skills/spec-orchestrator/SKILL.md`.
2. Inspect the 45 Check 23 findings on `uav-009` by running:
   `python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-009`
3. Fix cross-subsystem candidate metric binding in `factual_grounding_validator.py`:
   - Enforce distinguishing property tokens (e.g. `dive` for `maxDiveAngleDeg`, `pitch` for `gimbalPitchDownLimitDeg`). Prevent `maxDiveAngleDeg` from binding to lines describing gimbal travel, pitch, or yaw.
   - Forbid `MainBattery` metrics from binding to ground support equipment lines (`Pl40LauncherGSE`, `CompressorUnit`).
   - Ground legitimate user story quantities (kinetic energy, launch speed, coulomb counting, servo pulse widths) using schema/OEM text or appropriate design choice scoping.
4. Verify:
   - `python3 -m unittest tests/test_check23_factual_grounding_gate.py` (17/17 pass)
   - `python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-009` (exit code 0 across all checks)
   - `python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011` (exit code 0 across all checks)
5. Write handoff report to `.agents/worker_wp10b_tooling_1/handoff.md`.

## 2026-09-27T11:00:26Z
- Sender: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc
- Priority: HIGH
- Content: Heartbeat check (iteration 17): ~20 minutes since last progress update. Please report your progress on applying the validator refinements for the 5 diagnosed cases and running the verification checks.
- Action: Report current status and update progress.md.

## 2026-09-27T11:20:22Z
- Sender: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc
- Priority: HIGH
- Content: Heartbeat check (iteration 19): Please report your current status on the final verification run and handoff report.
- Action: Report current status and update progress.md.

## 2026-09-27T11:33:56Z
- Sender: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc
- Priority: HIGH
- Content: Heartbeat check: Did the Check 23 count command complete? What was the finding count on uav-009?
- Action: Report findings count and status.
