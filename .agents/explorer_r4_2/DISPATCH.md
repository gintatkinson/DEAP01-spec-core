## 2026-09-21T16:34:13Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2

Read the original user request at /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md (specifically the latest section ## 2026-09-21T16:32:10Z).
Read /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/PROJECT.md.

Your mission:
Investigate and survey `scripts/install_pipeline.sh`:
1. Trace the exact logic in `scripts/install_pipeline.sh` that scaffolds or updates downstream README.md files (e.g. `cat << 'EOF' > "$TARGET_DIR/README.md"` or similar).
2. Examine how repository role or type is currently detected or handled. Where is DOMAIN_DISTRIBUTION_TEMPLATE vs DOWNSTREAM_CUSTOMER_PROJECT determined or where can it be determined cleanly (e.g., inspecting directory, target repo name starting with DEAP- vs uav-, or command-line flags)?
3. Identify how the scaffolding logic must be split or branched to generate distinct READMEs for:
   - Domain Distribution Templates (DEAP-*): Declare DOMAIN_DISTRIBUTION_TEMPLATE, clean landing zone invariant in docs, customer onboarding command: `git clone <this-domain-template-remote-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`.
   - Customer Application Workspaces (uav-*): Declare DOWNSTREAM_CUSTOMER_PROJECT, zero circular clone commands, document project-specific commands: baseline verification (python3 scripts/verify_downstream_baseline.py), Step 0.0 Level 0 ingestion, and in-place tooling update: `bash scripts/install_pipeline.sh .`.
4. Check code fence and formatting compliance: ensure generated shell blocks have pure executable shell syntax without unescaped parentheses in comments.

Write your findings to /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2/report.md and write a structured handoff in /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2/handoff.md.

PROCEED
