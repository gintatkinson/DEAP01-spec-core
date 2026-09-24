## 2026-09-21T11:28:52Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

You are Worker WP2. Your working directory is `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1`.
You exclusively own and modify: `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`.
You MUST NOT modify any other files in the repository.

Context & Objectives:
Read `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md`, `/Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md`, `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_2/handoff.md`, and `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_3/handoff.md`.
Implement Work Package 2 (R2: Parameterized Domain Installer & Scaffolding) and Work Package 3 (R3: Robust Model & Schema Copying) in `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`:
1. WP3 (R3): Replace lines 283–289 with:
   ```bash
     mkdir -p "$TARGET_DIR/schema"
     if [ -d "$INSTALLER_ROOT/schema" ]; then
       cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
     fi
   ```
   Ensuring domain models and schemas are copied even if `$TARGET_DIR/schema` already exists, without creating nested `schema/schema/`.
2. WP2 (R2): Implement the 4-tier `$DOMAIN_REMOTE_URL` resolution hierarchy and update downstream `README.md` scaffolding around lines 478 and lines 594–645 per Explorer 2 handoff report.
3. In downstream `README.md`, ensure the customer project onboarding section embeds the domain repository's remote URL in the single self-contained command:
   ```bash
   git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
   ```
   with zero unquoted angle brackets and zero unescaped parentheses in comments.
4. Run verification tests (e.g. `python3 scripts/verify_downstream_baseline.py --no-domain` and sandbox installation tests) and document the commands and outputs.
5. Record your progress in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1/progress.md`, write your completion handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1/handoff.md`, send a message to parent with your handoff path, and finish.

PROCEED

## 2026-09-21T13:04:54Z

Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER

Role: Micro-Task Implementer
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1

Primary Commercial Toolchain Integration Context:
This platform explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. An auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Original User Request:
/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md

Task: Grounded Code Remediation in `scripts/install_pipeline.sh` (WP2 / Issue #363):
1. Add explicit `--domain-url <URL>` and `--domain-url=*` CLI parameter support to `scripts/install_pipeline.sh`:
   - Store in variable `DOMAIN_URL`.
   - Validate that if `--domain-url` is passed without an argument or followed by another flag, print an error and exit 1 (consistent with `--gitlab-url`, `--github-org`, etc.).
   - Update `show_help()` in `scripts/install_pipeline.sh` to document `--domain-url <URL>` ("Explicit remote URL for upstream domain template repository").
2. Decouple customer project provider (`$PROVIDER`, e.g. `gitlab`) from the host platform of the upstream domain template:
   - In the `DOMAIN_REMOTE_URL` resolution block (around line 610-634):
     - If `DOMAIN_URL` is set, prioritize it: `DOMAIN_REMOTE_URL="$DOMAIN_URL"`.
     - In the fallback logic (lines 626–633):
       DO NOT check `if [ "$PROVIDER" = "gitlab" ]` to synthesize a `gitlab.com` URL. The customer's provider is for customer tracking/repo, whereas upstream domain distribution templates canonicalize on GitHub.
       Ensure fallback resolves to GitHub:
       `DOMAIN_REMOTE_URL="https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git"`
       Never synthesize a non-existent `gitlab.com` URL for canonical domain templates like `DEAP-uas-infrastructure-safety`.
3. Verify that the customer onboarding command generated in downstream `README.md` files:
   ```bash
   git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
   ```
   resolves to the verified working domain remote URL (e.g. `https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git` when running with `--provider gitlab`).
4. Test your changes manually against a temporary directory (e.g. in `/tmp/test_install_domain_url/`) with:
   - `bash scripts/install_pipeline.sh /tmp/test_install_domain_url --provider gitlab --domain-name "DEAP-uas-infrastructure-safety"`
   - Verify `cat /tmp/test_install_domain_url/README.md` contains `git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline` and NOT `gitlab.com`.
   - Test `--domain-url https://example.com/custom-domain.git` and verify README reflects that URL.
5. Write your handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1/handoff.md` and report back via `send_message`.

PROCEED
