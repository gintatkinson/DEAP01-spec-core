## 2026-09-27T16:00:00Z

# Task Assignment: WP-03 Automated Regression & Scaffolding Test Verification

You are the Test Verification Worker for WP-03.
Target File Owned:
- `/Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py`

Tasks:
1. Update `tests/test_readme_scaffolding.py` per the specifications in `.agents/auditor_wp01/handoff.md` Section 5.2 and `implementation_plan.md` WP-03:
   - Update docstrings at lines 381 and 482 to reference Tier 2 Domain Distribution Templates and Tier 3 Customer Application Workspaces.
   - In `TestUpstreamCompilerReadme`, add:
     * `test_upstream_readme_section_1_heading_order_and_hierarchy`
     * `test_upstream_readme_three_tier_architecture_normalization`
     * `test_upstream_readme_pipeline_2_prompt_boundary_confinement`
   - In `TestDomainDistributionTemplateScaffolding.test_domain_template_scaffolding_explicit_role`:
     * Assert `content` contains `As a **Tier 2 Domain Distribution Template**`.
     * Assert `content` does not contain `Tier 1 Domain Distribution Template`.
   - In `TestCustomerWorkspaceScaffolding.test_customer_workspace_scaffolding_explicit_role`:
     * Assert `content` contains `As a **Tier 3 Customer Application Workspace**`.
     * Assert `content` does not contain `Tier 2 Customer Application Workspace`.
2. Run automated test suites:
   - `python3 -m unittest tests/test_readme_scaffolding.py` (assert exit code 0).
   - `python3 scripts/verify_downstream_baseline.py --no-domain` (assert exit code 0).
   - `python3 -m pytest tests/` (assert 0 failures).
3. Deliver complete report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp03/handoff.md`.
4. Notify orchestrator when done via send_message.


## 2026-10-03T18:55:17Z

# Task Assignment: WP-03 (Modular Package Implementation & Bootstrap Wrapper Refactoring)

You are the Lead Implementer for the DEAP01-spec-core repository refactoring project.

Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp03
Project Root: /Users/perkunas/jail/DEAP01-spec-core
Authoritative Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Authoritative Plan: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Architectural Specification: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_wp02/installer_architecture_spec.md
Legacy Installer: /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh

Execute WP-03 (Modular Package Implementation & Bootstrap Wrapper Refactoring):
1. Carefully read the architectural specification at `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_wp02/installer_architecture_spec.md`.
2. Implement the modular Python package `scripts/installer/`:
   - `scripts/installer/__init__.py`: Package metadata, version, and public exports.
   - `scripts/installer/cli.py`: CLI parser via `argparse`, full parity with all arguments/aliases.
   - `scripts/installer/metadata.py`: Git remote URL parser, provider detection, repo role resolution, upstream self-install refusal, metadata discovery/preservation manifest.
   - `scripts/installer/staging.py`: Atomic staging in tempfile, framework asset copying, landing zone .gitkeep enforcement, metadata restoration, atomic promotion.
   - `scripts/installer/tracker.py`: Tracker configuration update in codebase_rules.json and .gitlab-ci.yml, tracker label bootstrapping.
   - `scripts/installer/scaffolding.py`: Consolidated rules compilation, .env.template generation, dynamic downstream README.md generation, execution of scripts/scaffold_downstream_agents.py, git hooks setup, safety integrity test fixtures validation.
   - `scripts/installer/rollback.py`: Transactional snapshot manager, signal handling, automated atomic rollback.
3. Refactor `scripts/install_pipeline.sh`:
   - Reduce to a concise bootstrap wrapper (<150 lines, target ~45 lines).
   - Strict shell mode: `set -euo pipefail`.
   - Signal trapping: `trap cleanup EXIT ERR INT TERM`.
   - Validate prerequisites: Python 3.10+ and Git.
   - Delegate execution: `exec python3 -m scripts.installer.cli "$@"`.
   - Zero inline Python blocks (`python3 -c "..."`).
4. Strict Invariants to Enforce:
   - Pure Schema-Driven Compiler Invariant: Zero hardcoded customer domain concepts.
   - Zero Shell Injection: 100% list-style `subprocess.run(["git", ...], shell=False)`.
   - Zero Em Dash Invariant: ASCII `--` or `-` exclusively.
   - Strict Prohibition of Unit Tests: Do NOT create any `tests/`, `test_*.py`, or pytest suites.
5. Verify syntax and compilation of the new package:
   - Run `python3 -m py_compile scripts/installer/*.py` to ensure zero syntax errors.
   - Test CLI help output: `bash scripts/install_pipeline.sh --help` and `python3 -m scripts.installer.cli --help`.
6. Deliver your handoff report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp03/handoff.md` and send a message with the summary of changes.

PROCEED
