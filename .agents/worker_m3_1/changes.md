# Changes Report: Milestone M3 -- Comprehensive README Scaffolding Regression Test Suite

**Agent:** worker_m3_1  
**Target Milestone:** M3 (`tests/test_readme_scaffolding.py`)  
**Timestamp:** 2026-09-21T17:07:30Z  
**Exclusive Write Target:** `/Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py`  

---

## 1. Overview of Changes

Implemented a comprehensive, hermetic Python `unittest` regression test suite in `tests/test_readme_scaffolding.py` to prevent regressions in:
1. Upstream Spec Core Compiler README invariants (`DEAP01-spec-core/README.md`).
2. Tier 1 Domain Distribution Template scaffolding (`DEAP-*`).
3. Tier 2 Customer Application Workspace scaffolding (`uav-*`).
4. Universal shell code fence syntax validity (`bash -n`), comment parentheses hygiene, and angle-bracket placeholder quoting.

---

## 2. File Level Summary

### Created: `tests/test_readme_scaffolding.py` (573 lines)

Contains 4 test case classes with 17 total test methods:

#### Class 1: `TestUpstreamCompilerReadme` (5 tests)
- `test_upstream_readme_repository_classification`: Asserts `> **Repository Role:** `UPSTREAM_SPEC_CORE_COMPILER``.
- `test_upstream_readme_no_inline_python_scripts`: Asserts 0 matches for `python3 -c` or `python -c` in `README.md`, and 0 inline monkeypatching scripts targeting repository governance files (`AGENTS.md`).
- `test_upstream_readme_no_domain_clone_commands`: Asserts 0 matches for hardcoded domain clone commands (e.g. `DEAP-uas-infrastructure-safety`) in Section 5 installation instructions.
- `test_upstream_readme_compiler_focus_commands`: Verifies presence of compiler-focus commands (`python3 scripts/compile_sysml.py --compile`, `pytest`, `python3 scripts/verify_downstream_baseline.py --no-domain`, `bash scripts/install_pipeline.sh "<path-to-domain-template>"`, and `/tmp/deap_compiler` bootstrap propagation).
- `test_upstream_readme_anchor_links`: Parses all headings in `README.md`, computes GitHub-compatible anchor slugs, and verifies that every internal markdown link (`#...`) and relative local path (`docs/architecture/blueprints/...`) resolves cleanly on disk.

#### Class 2: `TestDomainDistributionTemplateScaffolding` (4 tests)
- `test_domain_template_scaffolding_explicit_role`: Executes `scripts/install_pipeline.sh <temp_dir> --role domain-template`, asserting `> **Repository Role:** `DOMAIN_DISTRIBUTION_TEMPLATE``, Section 1.1 Clean Landing Zone Invariant, and customer onboarding clone command.
- `test_domain_template_scaffolding_auto_detection`: Executes `scripts/install_pipeline.sh` on a directory named `DEAP-uas-infrastructure-safety` without `--role`, asserting auto-detection of `DOMAIN_DISTRIBUTION_TEMPLATE`.
- `test_domain_template_scaffolding_clean_landing_zones`: Asserts Section 1.1 documents clean landing zones for `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` with `.gitkeep` files.
- `test_domain_template_scaffolding_customer_clone_command_zero_sibling_dependencies`: Asserts that the customer onboarding clone command operates strictly with zero sibling path dependencies (`../...`).

#### Class 3: `TestCustomerWorkspaceScaffolding` (5 tests)
- `test_customer_workspace_scaffolding_explicit_role`: Executes `scripts/install_pipeline.sh <temp_dir> --role customer-project`, asserting `> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT``, zero circular self-clone commands, and project-specific commands.
- `test_customer_workspace_scaffolding_auto_detection`: Executes `scripts/install_pipeline.sh` on a directory named `uav-011` without `--role`, asserting auto-detection of `DOWNSTREAM_CUSTOMER_PROJECT` and zero circular clone commands.
- `test_customer_workspace_scaffolding_no_circular_clone`: Verifies that no circular clone command exists anywhere in the customer workspace README.
- `test_customer_workspace_scaffolding_idempotence`: Runs `scripts/install_pipeline.sh` on a customer project, records `README.md`, executes in-place `bash scripts/install_pipeline.sh .`, and asserts `README.md` is preserved identically.
- `test_customer_workspace_legacy_circular_upgrade`: Plants a legacy circular `README.md` containing `DOWNSTREAM_APPLICATION_WORKSPACE` and `git clone ... .tmp-pipeline`, runs in-place update, and verifies that it is upgraded to `DOWNSTREAM_CUSTOMER_PROJECT` with zero circular clone commands and project-specific commands restored.

#### Class 4: `TestShellCodeFenceHygiene` (3 tests)
- `test_upstream_readme_shell_code_fences_hygiene`: Validates all 17 bash/sh code fences in `DEAP01-spec-core/README.md` with `bash -n`, asserts zero unescaped parentheses in `#` comments, and asserts zero unquoted angle-bracket placeholders in executable command lines.
- `test_scaffolded_domain_template_shell_hygiene`: Validates all bash/sh code fences in scaffolded Domain Distribution Template `README.md` with `bash -n`, comment hygiene, and placeholder quoting.
- `test_scaffolded_customer_workspace_shell_hygiene`: Validates all bash/sh code fences in scaffolded Customer Workspace `README.md` with `bash -n`, comment hygiene, and placeholder quoting.

---

## 3. Design Decisions & Verification
- **Hermeticity**: All dynamic installer tests run in `tempfile.TemporaryDirectory()`, creating zero test workspace directories inside the repository.
- **Pure Schema-Driven Invariant**: Zero hardcoded domain concepts.
- **Toolchain Integration Context**: Adheres to MATLAB / Simulink / Stateflow / Embedded Coder citations in test module documentation.
- **Verification Results**:
  - `python3 -m unittest discover tests`: 40 tests passed in 16.323s.
  - `python3 scripts/verify_downstream_baseline.py --no-domain`: all 30 checks passed cleanly.
