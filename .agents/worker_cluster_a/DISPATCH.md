# Dispatch for Cluster A Implementer (R2: Positive AST Provenance & Anti-Regex Hardening)

## Objective
Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_a
Original Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Triage Report: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/triage_report.md

Exclusive Write Ownership:
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py`
- `skills/schema-specification-engineering/SKILL.md`
- `tests/test_factual_grounding_validator.py`
- `tests/test_ast_manifest_dispatch_contracts.py`

Tasks:
1. Remediate #378 and #376:
   - Eliminate line-level negative regex exemption `_has_epistemic_exemption()` and `EPISTEMIC_EXEMPTION_PATTERN` in `factual_grounding_validator.py`.
   - Remove the exemption bypass that allowed any line containing `[TIER-3]`, `(TIER-3)`, or `Declared Assumption` to evade numeric and structural grounding.
   - Enforce positive closed-world AST provenance against SysML v2 AST nodes.
2. Remediate #364:
   - Harden `_check_numeric_quantities` in `factual_grounding_validator.py` so numbers in Mermaid sequence diagrams, statecharts, flowcharts, and code blocks do not unconditionally bypass numeric grounding.
3. Remediate #377:
   - Update `skills/schema-specification-engineering/SKILL.md` to mandate projecting and injecting a typed parameter dictionary AST from `.pipeline/schema.sysml` into subagent prompts.
4. TDD Verification:
   - Write unit tests in `tests/test_factual_grounding_validator.py` and `tests/test_ast_manifest_dispatch_contracts.py`.
   - Verify `pytest tests/test_factual_grounding_validator.py` and `pytest tests/test_ast_manifest_dispatch_contracts.py` pass with 100% pass rate.
   - Confirm zero regressions in existing tests.
5. Provide handoff report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_a/handoff.md`.

PROCEED
