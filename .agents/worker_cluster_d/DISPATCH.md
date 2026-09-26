## 2026-09-26T19:52:11Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_d
Original Request Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan Path: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Triage Report Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/triage_report.md
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

You are the Cluster D Implementer (R5: Synthetic Mock Elimination in Safety & Parity Tests).
Your mission is to remediate issues #360, #349, and #286:

Exclusive File Ownership:
- `scripts/compile_sysml.py`
- `tests/test_compile_sysml_gate.py`
- `tests/test_cross_document_diagram_parity.py`
- `tests/test_check23_factual_grounding_gate.py`
- `tests/fixtures/safety/`

Detailed Scope:
1. Issue #360 (compile_sysml.py Phase Gate Guard & Capability compilation):
   - In `scripts/compile_sysml.py`, install a Phase Gate Guard on `forward_sync_sysml_to_specs()` ensuring it does not overwrite downstream landing zones (`docs/features/`, `docs/use-cases/`, `docs/user-stories/`) unless `--force` is explicitly passed or Phase 0 is verified.
   - Restore formal `SysMLCapabilityDef` parsing and compilation to Epics.
2. Issue #349 (Eliminate synthetic in-memory string mocks in diagram parity tests):
   - Establish active test suite `tests/test_cross_document_diagram_parity.py` (migrated from `archive/unit_tests_legacy/test_cross_document_diagram_parity.py`).
   - Purge synthetic in-memory mock strings and hardcoded domain concepts (`SwarmC2GroundStation`, `AlphaPlatform`, `WarheadModule`, etc.).
   - Wire tests to consume genuine schema fixtures from `tests/fixtures/` and canonical units from `skills/spec-conops-engineering/resources/`.
3. Issue #286 (Eliminate synthetic mocks in core safety validation):
   - Establish active test suite `tests/test_check23_factual_grounding_gate.py` (migrated from `archive/unit_tests_legacy/test_check23_factual_grounding_gate.py`).
   - Eliminate synthetic in-memory string mocks and tempfile mocks in safety tests.
   - Use persistent test fixtures under `tests/fixtures/safety/` to enforce closed-world model verification, testing that citation fraud and ungrounded claims are rejected.
4. TDD Verification:
   - Run tests:
     `python3 -m pytest tests/test_compile_sysml_gate.py tests/test_cross_document_diagram_parity.py tests/test_check23_factual_grounding_gate.py`
     Ensure 100% pass rate.
   - Confirm zero regressions in existing tests.
5. Report:
   - Document all changes and verification outputs in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_d/handoff.md`.
   - Send a message to the orchestrator when finished.

PROCEED
