## 2026-09-21T16:55:11Z
Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m2_1

Read the original user request at /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md (specifically section ## 2026-09-21T16:32:10Z § R2 and § R3).
Read /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/handoff.md and /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/changes.md.

Objective: Perform independent code and logic review of `scripts/install_pipeline.sh` following Milestone 2 changes:
1. Verify dynamic repository role detection:
   - `-r, --role ROLE` option parsing and validation.
   - Clean auto-detection logic for `DOMAIN_DISTRIBUTION_TEMPLATE` (DEAP-*) vs `DOWNSTREAM_CUSTOMER_PROJECT` (uav-* or customer projects).
2. Verify distinct README scaffolding:
   - For `DOMAIN_DISTRIBUTION_TEMPLATE`: role declaration, clean landing zone invariant documentation, customer onboarding clone command (`git clone "${DOMAIN_REMOTE_URL}" ./.tmp-pipeline && bash ...`), and in-place domain update command.
   - For `DOWNSTREAM_CUSTOMER_PROJECT`: role declaration, zero circular clone commands, local verification (`verify_downstream_baseline.py`), Level 0 ingestion, and in-place tooling update.
   - Both variants preserve Operator Prompt Catalog.
3. Verify README regeneration condition: ensures compliant READMEs are not overwritten repeatedly.
4. Run verification tests:
   `bash -n scripts/install_pipeline.sh`
   `python3 -m unittest discover tests`
   `python3 scripts/verify_downstream_baseline.py --no-domain`
5. Issue verdict: APPROVE or REQUEST_CHANGES.

Write your review to /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m2_1/review.md and handoff in /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m2_1/handoff.md.

PROCEED
