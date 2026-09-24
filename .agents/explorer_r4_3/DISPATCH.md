## 2026-09-21T16:34:14Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3

Read the original user request at /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md (specifically the latest section ## 2026-09-21T16:32:10Z).
Read /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/PROJECT.md.

Your mission:
Investigate and survey verification and testing facilities:
1. Examine `scripts/verify_downstream_baseline.py`: what checks does it run? Does it check README syntax, code fences, unescaped parentheses, or repository classifications? How does `--no-domain` behave?
2. Examine existing test files in `tests/` (e.g., `tests/test_install_pipeline.py`, `tests/test_domain_url_synthesis.py`, etc.) for patterns in testing `install_pipeline.sh` and README generation.
3. Formulate a test strategy and outline new tests (e.g., in `tests/test_readme_scaffolding.py`):
   - Verifying DEAP01-spec-core/README.md has no inline python scripts or domain clone commands.
   - Verifying distinct README generation for DOMAIN_DISTRIBUTION_TEMPLATE vs DOWNSTREAM_CUSTOMER_PROJECT.
   - Verifying no circular clone commands in customer workspace README.
   - Verifying all generated shell code fences parse cleanly without unescaped parentheses in comments.

Write your findings to /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/report.md and write a structured handoff in /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/handoff.md.

PROCEED
