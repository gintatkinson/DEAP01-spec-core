## 2026-09-24T18:32:00Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/adversarial-code-auditor/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

You are the Adversarial Code Auditor. Your working directory is `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6`.
Your task is to conduct an adversarial code audit on the downstream onboarding and rule-ingestion pipeline adhering to `skills/adversarial-code-auditor/SKILL.md` across 5 pillars, and produce a verified, fully compliant 7-section defect dossier diagnosing:
1. Token-conservation bias triggers in current prompt templates.
2. Lack of an installation-time consolidated governance bundle (`.pipeline/ACTIVE_RULES_BUNDLE.md`).
3. Fragility of open-ended folder directives across downstream customer workspaces.

Scope to inspect:
- `scripts/install_pipeline.sh`
- `scripts/scaffold_downstream_agents.py`
- `tests/test_readme_scaffolding.py`
- `README.md`
- `rules/` directory (note the 21 rules)
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` (header ## 2026-09-24T15:26:00Z)
- `/Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md`

Output Requirements:
- Follow the 7-section defect dossier skeleton in `skills/adversarial-code-auditor/SKILL.md` Section 2 exactly:
  - Section 1: Context and References with <!-- test-target: tests/test_readme_scaffolding.py --> and the 4 bullets (**File**:, **Pillar**:, **Symptom**:, **Test-Target**:).
  - Section 2: Root Cause Analysis with exactly 5 numbered "Why ...? Because ...".
  - Section 3: Correctness Analysis with exact file:line citations and invariant violations.
  - Section 4: Valid Mermaid sequence/class diagram illustrating the failure flow (must pass offline check 7 via parity_auditor).
  - Section 5: Affected Callers / Downstream Impact.
  - Section 6: Proposed Correction with code block.
  - Section 7: Relationship to Existing Issues.
  - Section Audit Source with SEVERITY: Important and FILE_LOCATION: scripts/install_pipeline.sh:<line-range>.
- Run validation check via `python3 scripts/file_defect.py --body-file /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md --dry-run --title "Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle"`.
- If defects or integrity violations are discovered, defect reports must be prepared for filing via `gh issue create` and `glab issue create` / `python3 scripts/file_defect.py`.
- Save the final verified dossier to `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md`.
- Write your completion report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/handoff.md`, send message to parent coordinator with dossier path and verification confirmation.

PROCEED
