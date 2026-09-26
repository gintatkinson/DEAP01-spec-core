# Dispatch for Cluster C Implementer (R4: Baseline Gate Masking & SSOT Parity)

## Objective
Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_c
Original Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Triage Report: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/triage_report.md

Exclusive Write Ownership:
- `scripts/verify_downstream_baseline.py`
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/architecture_viewpoint_validator.py`
- `README.md`
- `tests/test_conops_and_mission_intent_validators.py`
- `tests/test_baseline_fail_closed.py`
- `tests/test_dual_schema_parity.py`
- `tests/test_architecture_viewpoint_validator.py`

Tasks:
1. Remediate #375 (Checks 17, 20, 23 Green Test Trap):
   - In `scripts/verify_downstream_baseline.py`, modify Check 17 (Safety Integrity), Check 20 (WBS Suite), and Check 23 (Factual Grounding) so that when `allow_missing_specs=False` (or in strict downstream customer mode), missing specification directories or models fail closed with an error code 1 rather than silently returning exit code 0.
2. Remediate #365 (Gate 30 silent success on missing architecture corpus):
   - In `architecture_viewpoint_validator.py:1240-1248` and `scripts/verify_downstream_baseline.py:3432`:
     * Remove the unconditional bypass where `allow_missing_specs=True` causes Gate 30 to return empty findings and claim success when 0 architecture specs exist.
     * Enforce fail-closed behavior when the architecture corpus is absent.
3. Remediate #366 (Dual-schema SSOT parity gate):
   - In `scripts/verify_downstream_baseline.py`, implement Check 31 (Dual-Schema SSOT Parity Gate): if both `schema/*.sysml` and `.pipeline/schema.sysml` exist, verify that they are identical in AST definitions (`part def`, `port def`, `action def`, `item def`), preventing silent model drift.
4. Remediate #362 (Gate 26 test harness missing from tests/):
   - Restore `test_conops_and_mission_intent_validators.py` to `tests/` (from `archive/unit_tests_legacy/test_conops_and_mission_intent_validators.py` if present) ensuring it passes cleanly with `python3 -m unittest tests.test_conops_and_mission_intent_validators` without synthetic mock violations.
5. Remediate #361 (Initialization sequence prompt catalog link):
   - In `README.md:303-314`, add Step 6 to the Mandatory Agent Initialization Sequence directing agents to Section 9 (Operator Prompt Catalog).
6. TDD Verification:
   - Run tests:
     `python3 -m unittest tests.test_conops_and_mission_intent_validators`
     `python3 -m pytest tests/test_baseline_fail_closed.py tests/test_dual_schema_parity.py tests/test_architecture_viewpoint_validator.py`
   - Run `python3 scripts/verify_downstream_baseline.py --no-domain` to verify zero baseline regressions.
7. Provide handoff report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_c/handoff.md`.

PROCEED
