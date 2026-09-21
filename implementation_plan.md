# Implementation Plan: Domain Template URL Synthesis Remediation & Downstream Propagation

## Goal Description
Execute an adversarial code audit on `scripts/install_pipeline.sh` for the domain template URL synthesis bug, submit the verified 7-section defect dossier upstream via `python3 scripts/file_defect.py`, implement the verified fix across the compiler and downstream templates, and verify clean propagation and remote synchronization.

## User Authorization
> **Continuous Execution Authorized:** The user prompt explicitly provides "PROCEED", authorizing continuous execution through all documented work packages, subagent dispatches, verification tests, and git pushes.

---

## Proposed Changes

### Work Package 1: Adversarial 5-Pillar Code Audit & Defect Submission (R1)
- **Target Files / Scripts:**
  - `scripts/install_pipeline.sh` (lines 626–633)
  - `scripts/file_defect.py`
  - Defect payload generation under `.agents/auditor_wp1_1/`
- **Action Details:**
  1. Dispatch an adversarial code audit subagent (`teamwork_preview_critic` or `teamwork_preview_auditor`) adhering to `skills/adversarial-code-auditor/SKILL.md`.
  2. Perform a 5-pillar forensic audit on the fallback logic in `scripts/install_pipeline.sh` (lines 626–633) where the script synthesizes a `gitlab.com` URL for upstream domain repositories (`DEAP-uas-infrastructure-safety`) when the downstream project provider is GitLab (`$PROVIDER=gitlab`).
  3. Validate root cause: Conflating target customer project issue tracker / git host (`$PROVIDER`) with the host platform of the upstream domain template repository.
  4. Generate the compliant 7-section defect dossier and verify it offline against all Step D checks (including Mermaid syntax check 7).
  5. File the defect upstream using:
     `python3 scripts/file_defect.py --repo gintatkinson/DEAP01-spec-core --title "Tooling Bug: install_pipeline.sh synthesizes non-existent GitLab URLs for GitHub domain templates" --body-file <payload_path> --label "bug"`

### Work Package 2: Grounded Code Remediation in scripts/install_pipeline.sh (R2)
- **Target Files:**
  - `scripts/install_pipeline.sh`
- **Action Details:**
  1. Add explicit `--domain-url <URL>` (and `--domain-url=*`) CLI parameter support with clear error handling for missing argument.
  2. Decouple the customer project's provider (`$PROVIDER`, e.g. `gitlab`) from the host platform of the upstream domain template repository.
  3. Support `--domain-url` priority in `DOMAIN_REMOTE_URL` resolution:
     - If `--domain-url` is explicitly provided, use it directly.
     - If `REMOTE_URL` is set and does not contain `DEAP01-spec-core`: use `REMOTE_URL`.
     - Else if `INSTALLER_REMOTE` is set and does not contain `DEAP01-spec-core`: use `INSTALLER_REMOTE`.
     - Else if `DOMAIN_PROJECT_NAME` corresponds to a known canonical domain template (e.g. `DEAP-uas-infrastructure-safety`), ensure it resolves to its authoritative GitHub host (`https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git`).
     - Under no circumstances synthesize a non-existent `gitlab.com` URL for an upstream GitHub domain template repository.
  4. Ensure the customer onboarding command generated in downstream `README.md` files always targets the verified working domain remote URL:
     ```bash
     git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
     ```
  5. Preserve all existing behaviors for Tier 2 customer project updates and existing options.

### Work Package 3: Downstream Propagation & Remote Synchronization (R3)
- **Target Repositories:**
  - Upstream Compiler: `/Users/perkunas/jail/DEAP01-spec-core`
  - Domain Distribution Template: `DEAP-uas-infrastructure-safety` (cloned into `/tmp/DEAP-uas-infrastructure-safety` or target work area, updated, verified, committed, and pushed to remote)
  - Customer Project: `/Users/perkunas/jail/uav-011` (updated with `scripts/install_pipeline.sh .`, verified, committed, and pushed to GitLab remote)
- **Action Details:**
  1. Dispatch context-isolated worker to propagate the updated `scripts/install_pipeline.sh` to `DEAP-uas-infrastructure-safety`.
  2. In `DEAP-uas-infrastructure-safety`, run the updated installer to scaffold/update its `README.md`, ensuring the onboarding command targets `https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git`. Commit and push to remote.
  3. In `/Users/perkunas/jail/uav-011`, run the updated installer: `bash scripts/install_pipeline.sh . --domain-url https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git`.
  4. Verify that `uav-011` retains its git remote origin (`https://gitlab.com/gintatkinson/uav-011.git`) and has a clean, correct status. Commit and push to remote GitLab repository.
  5. Verify that `git diff origin/main` is completely empty and pushed across all affected repositories.

### Work Package 4: Full Test Verification & Conformance Gates
- **Target Files:**
  - Create regression unit tests in `tests/test_domain_url_synthesis.py`
  - Run `python3 scripts/verify_downstream_baseline.py --no-domain`
- **Action Details:**
  1. Add unit/regression tests verifying:
     - `install_pipeline.sh --provider gitlab` with a domain template resolves to GitHub and does NOT synthesize a non-existent `gitlab.com` URL.
     - `install_pipeline.sh --domain-url https://custom.domain.org/template.git` correctly sets `DOMAIN_REMOTE_URL`.
     - Downstream `README.md` onboarding command contains the expected git clone command with zero unescaped parentheses or unquoted angle brackets.
  2. Execute `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all 30 checks pass cleanly with exit code 0.

---

## Subagent Delegation Strategy
All implementation and testing is delegated to context-isolated subagents in accordance with `.agents/AGENTS.md`:
1. **Adversarial Auditor (`teamwork_preview_auditor` / `teamwork_preview_critic`)**:
   - Executes `skills/adversarial-code-auditor/SKILL.md`.
   - Performs 5-pillar audit on lines 626-633 of `scripts/install_pipeline.sh`.
   - Validates dossier schema and submits via `python3 scripts/file_defect.py`.
2. **Worker 1 (`teamwork_preview_worker`)**:
   - Implements `--domain-url` and provider decoupling in `scripts/install_pipeline.sh`.
3. **Worker 2 (`teamwork_preview_worker`)**:
   - Implements regression test suite `tests/test_domain_url_synthesis.py`.
4. **Worker 3 (`teamwork_preview_worker`)**:
   - Propagates installer to `DEAP-uas-infrastructure-safety` and `uav-011`, runs verification, commits, and pushes.
5. **Reviewers & Challengers (`teamwork_preview_reviewer`, `teamwork_preview_challenger`)**:
   - Verify code quality, test coverage, and baseline conformance.

---

## Verification Plan

### Automated Tests
- Run regression tests:
  `python3 -m unittest tests/test_domain_url_synthesis.py`
- Run baseline verification:
  `python3 scripts/verify_downstream_baseline.py --no-domain`
- Verify remote git state:
  `git -C /Users/perkunas/jail/DEAP01-spec-core diff origin/main`
  `git -C /Users/perkunas/jail/uav-011 diff origin/main`
