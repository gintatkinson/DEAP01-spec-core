## 2026-09-28T00:27:01+03:00

You are the Project Orchestrator for DEAP01-spec-core.

Identity: orchestrator
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_12
Parent Sentinel: e7232793-5a87-425f-bffa-24d699db1fa7
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Native Skill: skills/spec-orchestrator/SKILL.md
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Your task is to orchestrate the implementation and constitutional completion of Defect #381:
"[AUDIT] [README.md]: Universal multi-runtime initialization sequence misplaced inside vendor section 5.5"

## Requirements to Orchestrate:

### R1. Forensic Audit of README.md & Installer Scaffolding (WP-02)
- Deploy an adversarial auditor / explorer subagent to perform a forensic audit of `README.md` (Sections 5.5 through 5.8) and `scripts/install_pipeline.sh` README scaffolding logic.
- Document exact line numbers, AST heading hierarchy, and semantic traceability defects where the universal 6-step (0–7) post-install initialization sequence governing all AI agents is subordinated under vendor section 5.5 ("Setup for Google Antigravity").
- Specify the exact target diff for restructuring Section 5.

### R2. Relocate Universal Agent Governance & Initialization Sequence to Section 5.6 (WP-03)
- Restructure `README.md`:
  * Elevate Section 5.6 to: `### 5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)`.
  * Move the full post-install initialization sequence (steps 0 through 7: Repository Role & Scope Detection, Karpathy Compliance, Constitution Ingestion, Project Skills, Governance Rules, Platform Profile, Tracker Label Bootstrapping, and Transition to Operator Prompt Catalog) and the repository role classification taxonomy into Section 5.6.
  * Update Section 5.5 (`### 5.5 Setup for Google Antigravity`) to focus strictly on Antigravity native configuration discovery and provide an explicit pointer directing operators/agents to follow Section 5.6.
  * Update Section 5.7 (`### 5.7 Setup for Claude Code`) and Section 5.8 (`### 5.8 Setup for Cursor / Windsurf / Cascade`) to cross-reference Section 5.6.
- Check and update downstream README scaffolding in `scripts/install_pipeline.sh` if Section 5 scaffolding templates are generated for downstream repositories.

### R3. Test Suite Hardening & Regression Verification (WP-04)
- Add comprehensive regression tests in `tests/test_readme_scaffolding.py` verifying:
  1. Section 5.6 contains the Universal Agent Governance & Initialization Sequence.
  2. Section 5.5 does not subordinate the multi-agent initialization sequence under Antigravity, but cross-references Section 5.6.
  3. Sections 5.7 and 5.8 cross-reference the universal sequence in Section 5.6.
  4. Repository role classification taxonomy is cleanly positioned within Section 5.6.
- Run `python3 -m unittest tests/test_readme_scaffolding.py` and `pytest tests/test_readme_scaffolding.py`.
- Run `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all baseline checks pass cleanly with exit code 0.

### R4. Git Stage, Neutral Citation Commit & Remote Synchronization (WP-05)
- Verify commit neutrality via `python3 scripts/verify_commit_messages.py --head`.
- Stage all changes, commit using neutral issue citations:
  `git commit -m "docs(readme): relocate universal initialization sequence to section 5.6 (refs #381)"`
- Push to GitHub `origin/main` and verify that `git diff origin/main` returns exactly 0 bytes.

### R5. Issue Tracker Transition & Independent Victory Audit (WP-05)
- Post verification evidence comment to GitHub Issue #381 via `gh issue comment 381`.
- Transition issue label to `status:fixed-resolved` via `gh issue edit 381 --add-label "status:fixed-resolved"`.
- Dispatch an independent Victory Auditor subagent to verify the 0-byte remote diff, full test pass, and label update before declaring victory.

PROCEED

## Acceptance Criteria:
### Documentation & Governance Integrity
- [ ] Section 5.6 in `README.md` is titled `### 5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)` and contains the complete universal post-install initialization sequence (steps 0–7) and role classifications.
- [ ] Section 5.5 focuses on Google Antigravity native discovery and references Section 5.6.
- [ ] Section 5.7 (Claude Code) and Section 5.8 (Cursor / Windsurf / Cascade) explicitly reference Section 5.6.

### Automated Gates & Regression Tests
- [ ] `python3 -m unittest tests/test_readme_scaffolding.py` passes with exit code 0.
- [ ] `pytest tests/test_readme_scaffolding.py` passes with exit code 0.
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` passes all checks with exit code 0.
- [ ] `python3 scripts/verify_commit_messages.py --head` verifies zero issue-closing verbs.

### Git & Remote Synchronization
- [ ] Commit message strictly uses neutral citation `(refs #381)`.
- [ ] Changes pushed to `origin/main` with `git diff origin/main` returning exactly 0 bytes.
- [ ] Issue #381 transitioned to `status:fixed-resolved` with verification evidence comment.
