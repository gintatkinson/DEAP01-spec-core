# BRIEFING — 2026-09-21T13:08:00Z

## Mission
Remediate Issue #363 in scripts/install_pipeline.sh: add `--domain-url` and `--domain-name` CLI parameters, decouple provider from upstream domain host platform, prioritize DOMAIN_URL, and ensure fallback resolves to GitHub canonical domain templates without fabricating GitLab URLs.

## 🔒 My Identity
- Archetype: worker
- Roles: implementer, qa, specialist
- Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1
- Original parent: f3cf7ab0-b3f5-46dd-b0c6-a615bbe571ed
- Milestone: WP2 and WP3 execution

## 🔒 Key Constraints
- Exclusively own and modify: /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh
- MUST NOT modify any other files in the repository
- Pure Schema-Driven Compiler Invariant (Zero Hardcoded Domain Concepts)
- Upstream Distribution Template Clean Landing Zone Invariant
- Workspace-Relative Paths Invariant
- Genuine implementation: DO NOT CHEAT

## Current Parent
- Conversation ID: a40fd795-1e85-435d-9d9c-a1603074c664
- Updated: 2026-09-21T13:08:00Z

## Task Summary
- **What to build**: Support `--domain-url <URL>`, `--domain-url=*`, `--domain-name <NAME>`, `--domain-name=*` in `scripts/install_pipeline.sh`. Prioritize `DOMAIN_URL` in `DOMAIN_REMOTE_URL`. Decouple `$PROVIDER` from upstream domain template host so fallback points to GitHub (`https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git`) rather than non-existent `gitlab.com`. Update `show_help()`.
- **Success criteria**:
  1. `--domain-url` and `--domain-url=*` stored in `DOMAIN_URL`; missing argument raises error and exits 1.
  2. `--domain-name` and `--domain-name=*` stored in `DOMAIN_NAME`; missing argument raises error and exits 1.
  3. `show_help()` updated with `--domain-url` and `--domain-name`.
  4. If `DOMAIN_URL` is set, `DOMAIN_REMOTE_URL="$DOMAIN_URL"`.
  5. Fallback logic does not check `[ "$PROVIDER" = "gitlab" ]` to synthesize `gitlab.com` URL; fallback resolves to GitHub.
  6. Tested manually with `/tmp/test_install_domain_url/` under `--provider gitlab --domain-name "DEAP-uas-infrastructure-safety"` and `--domain-url`.
  7. `verify_downstream_baseline.py --no-domain` passes all checks.
- **Interface contracts**: implementation_plan.md, ORIGINAL_REQUEST.md
- **Code layout**: scripts/install_pipeline.sh

## Change Tracker
- **Files modified**: `scripts/install_pipeline.sh` (added `--domain-url`, `--domain-name`, decoupled `$PROVIDER` from upstream domain host, ensured GitHub fallback)
- **Build status**: PASS (`bash -n scripts/install_pipeline.sh` exit code 0; `verify_downstream_baseline.py --no-domain` exit code 0)
- **Pending issues**: None

## Quality Status
- **Build/test result**: PASS (all 30 baseline checks pass; manual sandbox tests with `--provider gitlab --domain-name "DEAP-uas-infrastructure-safety"` and `--domain-url` pass)
- **Lint status**: PASS (`bash -n` clean; zero unquoted angle brackets; zero unescaped parentheses in comments)
- **Tests added/modified**: Manual and automated tests against `/tmp/test_install_domain_url/` validating GitHub URL generation and no `gitlab.com` URLs

## Loaded Skills
- **Source**: /Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md
- **Local copy**: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp2_1/SKILL.md
- **Core methodology**: Feature-driven implementation lifecycle, TDD discipline, two-stage review gates, micro-task verification.

## Key Decisions Made
- Prioritize `DOMAIN_URL` if set: `DOMAIN_REMOTE_URL="$DOMAIN_URL"`.
- Support `--domain-name` to allow explicit domain project naming and satisfy test invocations.
- Fallback always resolves to GitHub (`https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git`).

## Artifact Index
- DISPATCH.md — Assignment instructions
- BRIEFING.md — Situational awareness
- progress.md — Heartbeat and progress tracking
- handoff.md — Final completion report

