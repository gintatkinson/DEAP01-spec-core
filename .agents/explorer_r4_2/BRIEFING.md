# BRIEFING — 2026-09-21T16:38:00Z

## Mission
Investigate and survey scripts/install_pipeline.sh README scaffolding logic, role detection, and branching for domain templates vs customer projects.

## 🔒 My Identity
- Archetype: explorer
- Roles: explorer, codebase investigator, pipeline installer surveyor
- Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_2
- Original parent: fc7b047c-fd31-4577-ab8b-b65f4c56c828
- Milestone: scripts/install_pipeline.sh investigation

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Scope restricted to scripts/install_pipeline.sh scaffolding & README logic
- Write only to .agents/explorer_r4_2/

## Current Parent
- Conversation ID: fc7b047c-fd31-4577-ab8b-b65f4c56c828
- Updated: not yet

## Investigation State
- **Explored paths**: `skills/feature-driven-implementation/SKILL.md`, `.pipeline/`, `scripts/install_pipeline.sh`, `scripts/scaffold_downstream_agents.py`, `scripts/verify_downstream_baseline.py`, `tests/test_domain_url_synthesis.py`, `ORIGINAL_REQUEST.md`, `orchestrator_4/PROJECT.md`
- **Key findings**:
  1. `scripts/install_pipeline.sh` hardcodes `DOWNSTREAM_APPLICATION_WORKSPACE` and generates monolithic README with circular customer clone command.
  2. Scaffolding trigger condition lines 517-520 requires `Customer Project Onboarding` or `.tmp-pipeline`, forcing unwanted re-scaffolding on customer workspaces unless made role-aware.
  3. Dynamic role detection can cleanly distinguish `DOMAIN_DISTRIBUTION_TEMPLATE` (via `--role domain-template`, target dir `DEAP-*`, remote `DEAP-*`, `--domain-name DEAP-*`, `--domain-url`) from `DOWNSTREAM_CUSTOMER_PROJECT` (via `--role customer-project`, target dir `uav-*`, remote `uav-*`, or fallback default).
  4. Branching design produces distinct Section 1 & Section 3 in README: domain template gets clean landing zone docs + customer onboarding command; customer project gets zero circular clone commands + project-specific verification, ingestion, and in-place update commands.
  5. Check 14 in `verify_downstream_baseline.py` passes for both variants because both include "Operator Prompt Catalog".
- **Unexplored areas**: None within assigned scope.

## Key Decisions Made
- Completed survey of `scripts/install_pipeline.sh` and formulated comprehensive branching architecture.
- Authored detailed findings in `report.md` and structured 5-component handoff in `handoff.md`.

## Artifact Index
- `DISPATCH.md` — incoming dispatch message
- `progress.md` — liveness heartbeat
- `report.md` — comprehensive survey and analysis report
- `handoff.md` — structured 5-component handoff report
