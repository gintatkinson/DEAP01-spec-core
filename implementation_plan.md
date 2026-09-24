# Implementation Plan: Downstream Onboarding Rule-Shortcutting Remediation & Consolidated Rule Bundling

## Goal Description
Execute an adversarial code audit on the downstream onboarding and rule-ingestion pipeline, file a formal verified defect report to `gintatkinson/DEAP01-spec-core`, and execute the debug protocol to implement and verify a deterministic fix preventing LLM agents from taking ingestion shortcuts.

Specifically:
1. **Adversarial Audit (R1)**: Diagnose token-conservation bias, open-ended directory reading failure modes, and lack of a single consolidated governance manifest using `skills/adversarial-code-auditor/SKILL.md`. Produce a 7-section defect dossier.
2. **Upstream Defect Submission (R2)**: File the defect via `python3 scripts/file_defect.py` to `gintatkinson/DEAP01-spec-core` with label `bug` and capture the issue URL/ID.
3. **Debug Protocol & Root-Cause Remediation (R3)**:
   - Update `scripts/install_pipeline.sh` to bundle all `rules/*.md` into `.pipeline/ACTIVE_RULES_BUNDLE.md` with table of contents and full rule bodies.
   - Update operator prompt catalogs in `scripts/install_pipeline.sh` so agents are directed to read `.pipeline/ACTIVE_RULES_BUNDLE.md` via `view_file` as their mandatory governance entry point.
   - Add regression tests in `tests/test_readme_scaffolding.py` to enforce bundle creation, rule completeness, and prompt references.
4. **Verification & Multi-Agent Gate**:
   - Verify unit test suite (`python3 -m unittest tests/test_readme_scaffolding.py` >= 18 tests passing).
   - Verify downstream baseline (`python3 scripts/verify_downstream_baseline.py --no-domain`).
   - Conduct Reviewer, Challenger, and Forensic Auditor verification.
   - Commit, push, and verify `git diff origin/<branch>` is clean.

## User Authorization
> **Continuous Execution Authorized:** The user prompt explicitly provides "PROCEED", fully authorizing continuous execution through all documented work packages, subagent dispatches, verification tests, and git pushes.

---

## Proposed Changes

### Work Package 1: Adversarial Audit & 5-Pillar Vulnerability Diagnosis (R1)
- **Target Files / Scope:**
  - `scripts/install_pipeline.sh`
  - `scripts/scaffold_downstream_agents.py`
  - `tests/test_readme_scaffolding.py`
  - `README.md`
- **Subagent Role:** Adversarial Code Auditor (`teamwork_preview_explorer` / `teamwork_preview_auditor`) armed with `skills/adversarial-code-auditor/SKILL.md`.
- **Deliverables:**
  - Complete 7-section defect dossier saved to `.agents/audit_r1/defect_dossier.md` analyzing:
    1. Executive Summary & Root Cause
    2. Vulnerability Surface & Mechanism of Failure (Token conservation bias, directory listing vs full reading, partial sampling)
    3. Concrete Reproducer & Failure Trace
    4. Blast Radius (Downstream customer onboarding across all platforms)
    5. Proposed Remediation Architecture (`.pipeline/ACTIVE_RULES_BUNDLE.md` generation, prompt catalog overhaul)
    6. Regression Test Strategy
    7. Upstream Defect Submission Payload

### Work Package 2: Upstream Defect Submission (R2)
- **Action Details:**
  - Dispatch subagent to execute `python3 scripts/file_defect.py` with:
    - `--repo gintatkinson/DEAP01-spec-core`
    - `--title "Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle"`
    - `--label "bug"`
    - `--body-file .agents/audit_r1/defect_dossier.md`
  - Verify issue creation and record the issue URL and number.

### Work Package 3: Active Governance Rule Bundling in `scripts/install_pipeline.sh` (R3.1)
- **Target Files:**
  - `scripts/install_pipeline.sh`
- **Action Details:**
  - Implement rule bundling function in `scripts/install_pipeline.sh`:
    - Iterate through all files in `rules/*.md`.
    - Generate `.pipeline/ACTIVE_RULES_BUNDLE.md` in the target repository.
    - Include header explaining the bundle is automatically compiled at installation time as the single-source-of-truth governance entry point.
    - Include a structured Table of Contents with anchors/links to each rule.
    - Include full unabridged content of each rule with clear delimiter sections (`## Rule: <filename>`).
    - Verify bundle creation for both domain template and customer project installations.

### Work Package 4: Operator Prompt Catalog Overhaul in `scripts/install_pipeline.sh` (R3.2)
- **Target Files:**
  - `scripts/install_pipeline.sh` (and `README.md` scaffolding functions)
- **Action Details:**
  - Update all generated operator prompts in downstream `README.md` (and related templates):
    - Replace open-ended `rules/` directory reads or isolated rule lists with an explicit instruction to execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md` as the very first step.
    - Emphasize that all governance rules are consolidated in this single file, ensuring 100% compliance in one read and eliminating shortcutting.

### Work Package 5: Regression Test Suite in `tests/test_readme_scaffolding.py` (R3.3)
- **Target Files:**
  - `tests/test_readme_scaffolding.py`
- **Action Details:**
  - Add comprehensive test cases verifying:
    1. `.pipeline/ACTIVE_RULES_BUNDLE.md` is generated during installation and contains all rule files from `rules/*.md`.
    2. Generated `README.md` operator prompts explicitly mandate reading `.pipeline/ACTIVE_RULES_BUNDLE.md`.
    3. No prompt templates point to isolated rule subsets.
    4. Run `python3 -m unittest tests/test_readme_scaffolding.py` and verify all tests pass cleanly (18/18+ tests).

### Work Package 6: Verification, Baseline, Multi-Agent Review, and Git Sync
- **Target Actions:**
  - Run `python3 scripts/verify_downstream_baseline.py --no-domain`.
  - Dispatch Reviewer (`teamwork_preview_reviewer`) to verify implementation quality and fidelity.
  - Dispatch Challenger (`teamwork_preview_challenger`) to stress-test rule bundling, edge cases (empty rules, missing directories, special characters), and prompt syntax.
  - Dispatch Forensic Auditor (`teamwork_preview_auditor`) for integrity verification.
  - Git commit, push, and remote sync verification (`git diff origin/main` clean).

---

## Subagent Delegation Strategy
1. **Adversarial Audit Subagent (`teamwork_preview_explorer`)**:
   - Working Directory: `.agents/audit_r1/`
   - Role: `Adversarial Code Auditor`
   - Task: Inspect scripts and prompt templates with `skills/adversarial-code-auditor/SKILL.md` and generate 7-section defect dossier.
2. **Defect Submitter Subagent (`teamwork_preview_worker`)**:
   - Working Directory: `.agents/defect_r2/`
   - Role: `Upstream Defect Reporter`
   - Task: Submit defect via `python3 scripts/file_defect.py` and record issue details.
3. **Implementation Worker Subagent (`teamwork_preview_worker`)**:
   - Working Directory: `.agents/worker_r3/`
   - Role: `Pipeline Engineer`
   - Task: Implement active rule bundling and prompt catalog update in `scripts/install_pipeline.sh`, and regression tests in `tests/test_readme_scaffolding.py`.
4. **Reviewer Subagents (`teamwork_preview_reviewer`)**:
   - Working Directory: `.agents/reviewer_1/`, `.agents/reviewer_2/`
   - Role: `Code Reviewer`
   - Task: Independent review of changes, test coverage, and documentation fidelity.
5. **Challenger Subagents (`teamwork_preview_challenger`)**:
   - Working Directory: `.agents/challenger_1/`, `.agents/challenger_2/`
   - Role: `Adversarial Verifier`
   - Task: Stress-test installation script, rule bundle generation, and prompt templates.
6. **Forensic Integrity Auditor (`teamwork_preview_auditor`)**:
   - Working Directory: `.agents/auditor_1/`
   - Role: `Forensic Integrity Auditor`
   - Task: Full integrity forensics verification against hardcoding, facade logic, and shortcuts.

---

## Verification Plan
### Automated Tests
- `python3 -m unittest tests/test_readme_scaffolding.py`
- `python3 scripts/verify_downstream_baseline.py --no-domain`
- `pytest tests/`
- Git remote diff: `git diff origin/main`
