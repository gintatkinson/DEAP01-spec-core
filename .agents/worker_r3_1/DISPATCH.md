## 2026-09-21T11:54:14Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

You are Worker R3_1. Your working directory is `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r3_1`.
You exclusively own and modify:
1. `/Users/perkunas/jail/DEAP01-spec-core/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py`
2. `/Users/perkunas/jail/DEAP01-spec-core/docs/OPERATOR_PROMPT_CATALOG.md`
You MUST NOT modify any other files in the repository.

Context & Objectives:
Read `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` and `/Users/perkunas/jail/DEAP01-spec-core/.agents/challenger_r2_2/handoff.md`.
Challenger R2_2 uncovered that when domain models (e.g. `schema/domain_model.sysml`) exist in a downstream repository, running `python3 scripts/verify_downstream_baseline.py --no-domain` fails with exit code 1:
`ERROR: Check 23 failed (Factual Grounding & Numeric Provenance Gate violations found): docs/OPERATOR_PROMPT_CATALOG.md:216: Critical citation fraud: cited source file 'schema/model.sysml' does not exist in workspace.`

Tasks:
1. In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py`:
   In `_is_excluded_spec_file()`, ensure non-specification developer guides and catalogs are excluded from factual grounding scanning:
   ```python
   if filename in ("OPERATOR_PROMPT_CATALOG.md", "JIRA_INTEGRATION_GUIDE.md", "README.md"):
       return True
   ```
2. In `docs/OPERATOR_PROMPT_CATALOG.md`:
   At line 216, reword the sentence so that it does not use file path patterns matching citation regex for non-existent models:
   ```markdown
   Extracting OEM Bill of Materials (BOM) and physical parameters into `schema/extracted/` and synthesizing canonical SysML v2 textual models (such as `model.sysml` in `schema/` or `.pipeline/schema.sysml`) is fully authorized under Check 23 (Factual Grounding & Numeric Provenance Gate) and serves as the mandatory precursor to executing the Step 0 compilation gate (`python3 scripts/compile_sysml.py --compile`).
   ```
3. Verification:
   - Run `python3 -m py_compile skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py` to ensure syntax is clean.
   - Run `python3 scripts/verify_downstream_baseline.py --no-domain` in `DEAP01-spec-core` (must exit 0).
   - In a temporary isolated directory (e.g. in `/tmp`), verify that running `verify_downstream_baseline.py --no-domain` with a dummy SysML file in `schema/` now passes Check 23 cleanly without citation fraud.
   - Verify that markdown code blocks in `docs/OPERATOR_PROMPT_CATALOG.md` have 0 unescaped parentheses in comments and 0 unquoted angle-bracket placeholders.
4. Record your progress in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r3_1/progress.md`, write your completion handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r3_1/handoff.md`, send a message to parent with your handoff path, and finish.

PROCEED
