## 2026-09-21T17:08:12Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m3_1

Read the original user request at /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md (specifically section ## 2026-09-21T16:32:10Z § Acceptance Criteria).
Read /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m3_1/handoff.md and /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m3_1/changes.md.
Read /Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py.

Objective: Perform independent code and quality review of `tests/test_readme_scaffolding.py`:
1. Verify the 4 test suites:
   - `TestUpstreamCompilerReadme`: verifies no inline python scripts, no domain clones, compiler-focused commands, and valid link resolution in `README.md`.
   - `TestDomainDistributionTemplateScaffolding`: verifies role declaration `DOMAIN_DISTRIBUTION_TEMPLATE`, clean landing zone invariant in docs, and single-line customer clone command without circular references.
   - `TestCustomerWorkspaceScaffolding`: verifies role declaration `DOWNSTREAM_CUSTOMER_PROJECT`, zero circular clone commands, project-specific baseline/ingestion commands, idempotence on repeat runs, and legacy circular README migration.
   - `TestShellCodeFenceHygiene`: validates pure shell execution via `bash -n`, zero unescaped parens in `#` comments, and zero unquoted angle brackets across all tiers.
2. Run verification tests:
   `python3 -m unittest discover tests`
   `python3 scripts/verify_downstream_baseline.py --no-domain`
3. Issue verdict: APPROVE or REQUEST_CHANGES.

Write your review to /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m3_1/review.md and handoff in /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m3_1/handoff.md.

PROCEED
