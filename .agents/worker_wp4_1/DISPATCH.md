## 2026-09-21T13:11:41Z

Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER

Role: Test Writer
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp4_1

Primary Commercial Toolchain Integration Context:
This platform explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. An auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Original User Request:
/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md

Task: Work Package 4 - Author Regression Test Suite for Domain Template URL Synthesis and CLI Parameter Support:
1. Create `tests/test_domain_url_synthesis.py` using standard `unittest` framework.
2. The test suite MUST test all aspects of Issue #363 / `scripts/install_pipeline.sh`:
   - `test_help_shows_domain_options`: Runs `bash scripts/install_pipeline.sh --help` and asserts `--domain-url` and `--domain-name` are documented.
   - `test_domain_url_missing_arg_fails`: Runs `bash scripts/install_pipeline.sh --domain-url` and `bash scripts/install_pipeline.sh --domain-url --provider gitlab` and asserts returncode == 1 with error on stderr.
   - `test_domain_name_missing_arg_fails`: Runs `bash scripts/install_pipeline.sh --domain-name` and asserts returncode == 1 with error on stderr.
   - `test_domain_remote_url_with_gitlab_provider_defaults_to_github`:
     Creates a temporary directory, runs `bash scripts/install_pipeline.sh <temp_dir> --provider gitlab --domain-name "DEAP-uas-infrastructure-safety"`.
     Reads `<temp_dir>/README.md`, asserts:
     - Contains `git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
     - Does NOT contain `gitlab.com` in any git clone or domain URL in README.md.
   - `test_explicit_domain_url_flag`:
     Creates a temporary directory, runs `bash scripts/install_pipeline.sh <temp_dir> --provider gitlab --domain-url https://example.com/custom/domain.git`.
     Reads `<temp_dir>/README.md`, asserts:
     - Contains `git clone https://example.com/custom/domain.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
   - `test_explicit_domain_url_equals_syntax`:
     Creates a temporary directory, runs `bash scripts/install_pipeline.sh <temp_dir> --domain-url=https://github.com/org/domain-repo.git`.
     Reads `<temp_dir>/README.md`, asserts:
     - Contains `git clone https://github.com/org/domain-repo.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
   - `test_target_origin_remote_preserved`:
     Creates a temporary directory with `git init` and `git remote add origin https://github.com/org/DEAP-uas-infrastructure-safety.git`, runs `bash scripts/install_pipeline.sh <temp_dir>`, asserts README uses `https://github.com/org/DEAP-uas-infrastructure-safety.git`.
3. All temporary directories must use `tempfile.TemporaryDirectory` or cleanup in `tearDown` so no test artifacts leak into the repo.
4. Run the tests via:
   `python3 -m unittest tests/test_domain_url_synthesis.py -v`
   and verify all tests pass cleanly.
5. Also run:
   `python3 scripts/verify_downstream_baseline.py --no-domain`
   and verify that baseline conformance remains green.
6. Write your handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp4_1/handoff.md` and report back via `send_message`.

PROCEED
