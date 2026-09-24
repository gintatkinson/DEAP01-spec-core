## 2026-09-21T16:49:07Z
Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1

Primary Commercial Toolchain Integration Context: All subagent dispatches for control law synthesis, safety statechart modeling, and DO-178C C/SPARK Ada code generation MUST cite MATLAB / Simulink / Stateflow / Embedded Coder as the primary commercial toolchain integration context.

Exclusive Write Ownership: You exclusively own `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`. You MUST NOT write to any other repository source or test file.

Inputs to read:
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` (specifically section ## 2026-09-21T16:32:10Z § R2 and § R3)
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2/handoff.md`
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2/report.md`
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/PROJECT.md`

Objective: Implement Milestone 2 (R2 and R3) in `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`:
1. Dynamic Repository Role Detection:
   - Support an explicit CLI argument: `-r, --role ROLE` (valid values: `domain-template`, `customer-project`, or full classification names `DOMAIN_DISTRIBUTION_TEMPLATE`, `DOWNSTREAM_CUSTOMER_PROJECT`).
   - If `--role` is not specified, auto-detect cleanly:
     * If the target repository name / basename starts with `DEAP-`, or git remote contains `DEAP-`, or `--domain-name` starts with `DEAP-`: classify as `DOMAIN_DISTRIBUTION_TEMPLATE`.
     * Otherwise (or if basename starts with `uav-`): classify as `DOWNSTREAM_CUSTOMER_PROJECT`.
2. Distinct README Scaffolding:
   - When target role is `DOMAIN_DISTRIBUTION_TEMPLATE` (DEAP-*):
     - Declare repository role as `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - Document the clean landing zone invariant in `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`.
     - Document the authoritative single-line customer onboarding command:
       ```bash
       git clone "${DOMAIN_REMOTE_URL}" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
       ```
     - Preserve Section 4 Operator Prompt Catalog.
   - When target role is `DOWNSTREAM_CUSTOMER_PROJECT` (uav-*):
     - Declare repository role as `DOWNSTREAM_CUSTOMER_PROJECT`.
     - Eliminate circular instructions telling the customer to clone from this project into itself.
     - Document project-specific operational commands:
       - Baseline verification: `python3 scripts/verify_downstream_baseline.py --no-domain`
       - Step 0.0 Level 0 OEM Ground Truth Ingestion:
         `python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema "schema/extracted/" --format markdown --out "schema/model.sysml"`
         and SysML compilation gate: `python3 scripts/compile_sysml.py --compile`
       - In-place tooling update: `bash scripts/install_pipeline.sh .`
     - Preserve Section 4 Operator Prompt Catalog.
3. Update README Regeneration Trigger Condition:
   - Ensure the check around lines 517–520 recognizes both `DOMAIN_DISTRIBUTION_TEMPLATE` and `DOWNSTREAM_CUSTOMER_PROJECT` README formats so that compliant READMEs are not unnecessarily overwritten.
4. Code Fence & Shell Hygiene:
   - Ensure all generated code fences contain pure valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.
5. Verification:
   - Test `install_pipeline.sh` in hermetic temporary directories using both `--role domain-template` and `--role customer-project` (and auto-detection).
   - Verify `python3 -m unittest discover tests` (including `tests/test_domain_url_synthesis.py`) passes cleanly.
   - Verify `python3 scripts/verify_downstream_baseline.py --no-domain` passes cleanly.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your changes report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/changes.md` and handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/handoff.md`.

PROCEED
