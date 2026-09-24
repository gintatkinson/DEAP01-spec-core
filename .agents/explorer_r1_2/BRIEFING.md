# BRIEFING — 2026-09-21T11:27:45Z

## Mission
Investigate Work Package 2 (R2: Parameterized Domain Installer & Scaffolding) focusing on scripts/install_pipeline.sh downstream README generation, REMOTE_URL detection, customer onboarding command parameterization, and deliver a concrete code diff and 5-component handoff report.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Codebase Researcher, Specification Analyst
- Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_2
- Original parent: f3cf7ab0-b3f5-46dd-b0c6-a615bbe571ed
- Milestone: R2 Investigation

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Do NOT modify any codebase or specification files
- Write progress and final handoff report exclusively to working directory

## Current Parent
- Conversation ID: f3cf7ab0-b3f5-46dd-b0c6-a615bbe571ed
- Updated: 2026-09-21T11:27:45Z

## Investigation State
- **Explored paths**: skills/feature-driven-implementation/SKILL.md, .pipeline/constitution.md, scripts/install_pipeline.sh (lines 160-250, 475-645, 1040-1133), README.md (lines 1-240), scripts/verify_downstream_baseline.py (lines 505-535, Check 14), implementation_plan.md, ORIGINAL_REQUEST.md.
- **Key findings**: Downstream README.md is scaffolded in scripts/install_pipeline.sh lines 475-645 and 1090-1099. Currently lacks an onboarding installation command. REMOTE_URL is detected from TARGET_DIR git remote at line 172. Heredoc at line 596 (cat << EOF) expands bash variables, while line 643 (cat << 'EOF') does not. Customer onboarding section must reside in the expanding block before line 643 with properly escaped backticks and robust fallback logic avoiding DEAP01-spec-core.
- **Unexplored areas**: None for Work Package 2.

## Key Decisions Made
- Parameterize DOMAIN_REMOTE_URL with four-tier resolution: (1) TARGET_DIR remote if not DEAP01-spec-core, (2) INSTALLER_ROOT remote if not upstream compiler, (3) detected namespace/project HTTPS URL, (4) synthesized provider/org/project fallback.
- Formatted customer onboarding command into Section 3 (with 3.1 Onboarding and 3.2 Agent Initialization) to preserve 1..5 numbering without altering 400+ lines in Section 4.
- Ensured zero unquoted angle-brackets and zero parentheses in comments.

## Artifact Index
- /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_2/progress.md — Liveness heartbeat and progress log
- /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_2/handoff.md — 5-component handoff report
