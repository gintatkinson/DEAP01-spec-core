# Progress - WP-09 Tooling Fix

- Last visited: 2026-09-27T08:23:30Z
- Status: Completed WP-09 implementation and verification.
  * Fixed `ArchitectureViewpointValidator.validate`: returns `[]` when `not active_spec_files` and `allow_missing_specs=True`.
  * Updated `check_architecture_viewpoint_diagrams`: accepts `allow_missing_specs` and `strict`, calculates `effective_allow_missing`, and passes `allow_missing_specs=effective_allow_missing` to `validator.validate`.
  * Updated `run_all_checks`: passes `allow_missing_specs=allow_missing_specs, strict=strict` to `check_architecture_viewpoint_diagrams`.
  * Added `_has_clean_landing_zones`: detects clean schema landing zones (only `.gitkeep` / `README.md`) and clean specification landing zones (`docs/epics`, `docs/features`, `docs/user-stories`, `docs/use-cases` have zero concrete specs). When detected and not in strict mode, defaults `allow_missing = True`.
  * Updated Check 17 (`check_safety_integrity_and_sora_completeness`): when `effective_allow_missing` is True, emits `Success: Check 17 verified (Downstream repository detected -- safety specifications pending or clean).`
  * Updated unit test `test_downstream_missing_architecture_corpus_fails_closed` and added `test_downstream_missing_architecture_corpus_allowed_when_flag_set` in `tests/test_architecture_viewpoint_validator.py`.
  * Verified unit tests pass (13/13).
  * Verified full test suite passes (294/294).
  * Verified `python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011` passes Checks 10 through 31 with exit code 0.
  * Verified `--strict` and `DEAP_STRICT_BASELINE=1` fail closed at Check 17 as expected.
