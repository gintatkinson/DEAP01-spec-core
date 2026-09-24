# BRIEFING — 2026-09-21T16:54:30Z

## Mission
Implement Milestone 2 (R2 and R3) in scripts/install_pipeline.sh: dynamic repository role detection, distinct README scaffolding for domain templates vs downstream customer projects, update regeneration trigger, and shell hygiene.

## 🔒 My Identity
- Archetype: implementer
- Roles: implementer, qa
- Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1
- Original parent: fc7b047c-fd31-4577-ab8b-b65f4c56c828
- Milestone: Milestone 2 (R2 and R3)

## 🔒 Key Constraints
- Exclusive Write Ownership: You exclusively own /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh. You MUST NOT write to any other repository source or test file.
- Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
- Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
- Clean Landing Zone Invariant
- Pure Schema-Driven Compiler Invariant
- Workspace-Relative Paths Invariant
- Commit Message Non-Closure Invariant
- Never hardcode test results or dummy implementations

## Current Parent
- Conversation ID: fc7b047c-fd31-4577-ab8b-b65f4c56c828
- Updated: 2026-09-21T16:54:30Z

## Task Summary
- **What to build**: Update scripts/install_pipeline.sh to support dynamic repository role detection (-r/--role flag and auto-detection), distinct README scaffolding for DOMAIN_DISTRIBUTION_TEMPLATE and DOWNSTREAM_CUSTOMER_PROJECT, fix README regeneration condition, ensure code fence hygiene.
- **Success criteria**: Tests in `tests/` pass, baseline verification passes, hermetic test runs pass with both roles.
- **Interface contracts**: /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/PROJECT.md
- **Code layout**: scripts/install_pipeline.sh

## Change Tracker
- **Files modified**: scripts/install_pipeline.sh (completed)
- **Build status**: PASS (23 tests passed, 30 checks passed, hermetic test suite passed)
- **Pending issues**: None

## Quality Status
- **Build/test result**: PASS
- **Lint status**: 0 violations, bash -n exit 0
- **Tests added/modified**: Hermetic validation passed across all roles

## Loaded Skills
- **Source**: /Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md
- **Local copy**: /Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md
- **Core methodology**: Agile feature implementation using serial subagent TDD execution discipline with two-stage review gates.

## Key Decisions Made
- [initial decision] Implement CLI argument parsing and auto-detection for repository role in install_pipeline.sh.
- [M2 decision] Structure README scaffolding into role-aware branches: DOMAIN_DISTRIBUTION_TEMPLATE and DOWNSTREAM_CUSTOMER_PROJECT.
- [M2 decision] Replace lines 517-520 with role-aware SHOULD_SCAFFOLD_README condition recognizing both compliant README formats.
- [M2 decision] Eliminate circular clone instructions in customer project scaffolding, adding downstream verification and Level 0 OEM Ground Truth ingestion commands instead.

## Artifact Index
- /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/DISPATCH.md - Dispatch instructions
- /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/BRIEFING.md - Working memory
- /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/progress.md - Heartbeat/liveness
- /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/changes.md - Changes report
- /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m2_1/handoff.md - Handoff report
