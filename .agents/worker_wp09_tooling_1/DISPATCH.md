## 2026-09-27T08:14:08Z
Execute view_file on skills/debug-protocol/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp09_tooling_1
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Parent Orchestrator ID: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc

Task: Execute WP-09 from approved implementation_plan.md: Tooling Fix for Check 30 and Clean Landing Zone Baseline Gating.

Problem Description:
Independent Victory Auditor 6 found that running baseline verification on unelaborated downstream workspace uav-011:
1. Terminated with exit code 1 at Check 17:
   `ERROR: Check 17 failed: Safety specification directory 'docs/safety/' is missing.`
2. When run with `--allow-missing-specs`, terminated with exit code 1 at Check 30:
   `ERROR: Check 30 failed (Architecture Viewpoint & Diagram Completeness violations found):`
   `- Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').`

Root Cause & Requirements:
1. In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/architecture_viewpoint_validator.py` around line 1240:
   Under `# If downstream and no specification documents found`:
   The validator unconditionally returned `RULE_CORPUS_MISSING` even when `allow_missing_specs=True`!
   Fix: Check `if allow_missing_specs: return []` before emitting `RULE_CORPUS_MISSING`.
2. In `scripts/verify_downstream_baseline.py` around line 3450 (`check_architecture_viewpoint_diagrams`):
   The function hardcodes `validator.validate(repo, allow_missing_specs=False, spec_only=True)` and does not take `allow_missing_specs` or `strict` as arguments.
   Fix: Update signature to `def check_architecture_viewpoint_diagrams(repo_root=None, allow_missing_specs=False, strict=False):`
   Calculate `effective_allow_missing = allow_missing_specs and not (strict or (os.environ.get("DEAP_STRICT_BASELINE", "").lower() in ("1", "true", "yes")))`
   Pass `allow_missing_specs=effective_allow_missing` into `validator.validate(...)`.
   In `run_all_checks` (line 3644), pass `allow_missing_specs=allow_missing_specs, strict=strict`.
3. In `scripts/verify_downstream_baseline.py` around line 3649:
   In `_run_verification`:
   Detect if the repository has clean landing zones (e.g. `schema/` has only `.gitkeep` / no concrete models, and `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` have only `.gitkeep` with zero concrete specifications). If so, and if `--strict` / `DEAP_STRICT_BASELINE` is not set, set `effective_allow_missing = True` (or default `allow_missing = True` for clean landing zone repositories).
4. In `scripts/verify_downstream_baseline.py` Check 17 (`check_safety_integrity_and_sora_completeness` lines 2084-2104):
   Ensure that if `effective_allow_missing` is True, a missing `docs/safety/` directory or a clean `docs/safety/` directory with only `.gitkeep`/`README.md` emits `Success: Check 17 verified (Downstream repository detected -- safety specifications pending or clean).` and returns without error.

Testing & Verification:
- Run `python3 -m unittest tests/test_architecture_viewpoint_validator.py` to ensure unit tests pass.
- Run `python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011` directly (with NO extra flags) and verify Checks 10 through 31 pass with exit code 0.
- Report all changes and verification results in `.agents/worker_wp09_tooling_1/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED
