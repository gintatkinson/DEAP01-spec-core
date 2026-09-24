## 2026-09-21T17:00:40Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m3_1

Primary Commercial Toolchain Integration Context: All subagent dispatches for control law synthesis, safety statechart modeling, and DO-178C C/SPARK Ada code generation MUST cite MATLAB / Simulink / Stateflow / Embedded Coder as the primary commercial toolchain integration context.

Exclusive Write Ownership: You exclusively own `/Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py`. You MUST NOT modify any other file in the repository.

Inputs to read:
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` (specifically section ## 2026-09-21T16:32:10Z § Acceptance Criteria)
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/handoff.md`
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/report.md`
- `/Users/perkunas/jail/DEAP01-spec-core/tests/test_domain_url_synthesis.py` (as reference for hermetic testing with tempfile.TemporaryDirectory)

Objective: Implement the comprehensive regression test suite in `/Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py` using Python's `unittest` framework:
1. `TestUpstreamCompilerReadme`:
   - `test_upstream_readme_no_inline_python_scripts`: verifies 0 matches for `python3 -c` or inline monkeypatching scripts in `README.md`.
   - `test_upstream_readme_no_domain_clone_commands`: verifies 0 matches for hardcoded domain clone commands (e.g. `DEAP-uas-infrastructure-safety`) in Section 5 installation instructions.
   - `test_upstream_readme_compiler_focus_commands`: verifies presence of `compile_sysml.py`, `pytest`, `verify_downstream_baseline.py --no-domain`, and propagation commands (`install_pipeline.sh "<path-to-domain-template>"` and `/tmp/deap_compiler`).
   - `test_upstream_readme_anchor_links`: verifies all internal markdown links and anchor slugs in `README.md` resolve cleanly.
2. `TestDomainDistributionTemplateScaffolding`:
   - Runs `scripts/install_pipeline.sh` on a temporary directory with `--role domain-template` and with auto-detection on `DEAP-*` name.
   - Asserts `> **Repository Role:** `DOMAIN_DISTRIBUTION_TEMPLATE``.
   - Asserts Clean Landing Zone Invariant in Section 1.1.
   - Asserts single-line customer clone command (`git clone "${DOMAIN_REMOTE_URL}" ./.tmp-pipeline && bash ...`).
3. `TestCustomerWorkspaceScaffolding`:
   - Runs `scripts/install_pipeline.sh` on a temporary directory with `--role customer-project` and with auto-detection on `uav-*` name.
   - Asserts `> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT``.
   - Asserts ZERO circular self-clone commands (`git clone.*\.tmp-pipeline` must NOT be present).
   - Asserts project-specific commands: baseline verification (`verify_downstream_baseline.py`), Level 0 ingestion, and in-place update (`bash scripts/install_pipeline.sh .`).
   - Asserts idempotence: running `install_pipeline.sh .` a second time preserves the README.
   - Asserts legacy circular customer README upgrade: legacy circular README is detected and upgraded.
4. `TestShellCodeFenceHygiene`:
   - Extracts all bash/sh code fences from `README.md` and scaffolded templates.
   - Validates each with `bash -n` via subprocess.
   - Verifies zero unescaped parentheses in `#` comments inside bash fences.
   - Verifies zero unquoted angle brackets (`<...>`) in executable bash commands.
5. Verification:
   - Run `python3 -m unittest discover tests` and ensure all tests pass (23 previous + new tests).
   - Run `python3 scripts/verify_downstream_baseline.py --no-domain` and ensure all 30 checks pass.
