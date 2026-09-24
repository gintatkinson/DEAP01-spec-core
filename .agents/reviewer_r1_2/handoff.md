# Review & Adversarial Critic Report: scripts/install_pipeline.sh & README.md

**Agent:** Reviewer 2 (`reviewer_r1_2`)  
**Roles:** Reviewer, Critic  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_r1_2`  
**Target Milestone:** Two-Tier Architecture Alignment & Robust Model Copying Review  
**Date:** 2026-09-21  
**Verdict:** **APPROVE**  

---

## Review Summary

**Verdict**: **APPROVE**  
**Integrity Status**: CLEAN (Zero integrity violations found; no hardcoded test results, facade logic, or verification bypasses).  
**Risk Level**: LOW  

---

## 1. Observation

Direct examination and empirical testing of the modified files and test suite yielded the following:

### 1.1 `scripts/install_pipeline.sh` Schema Copying Logic (R3)
Lines 283–286 of `scripts/install_pipeline.sh`:
```bash
  mkdir -p "$TARGET_DIR/schema"
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
  fi
```
- **Observed Behavior:**
  - Before change: `if [ ! -e "$TARGET_DIR/schema" ]` evaluated to `false` whenever `$TARGET_DIR/schema` pre-existed (e.g. holding `.gitkeep` from repository initialization), skipping domain model copying completely.
  - After change: `mkdir -p "$TARGET_DIR/schema"` guarantees target directory existence, and `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` copies all contents of `$INSTALLER_ROOT/schema/` directly into `$TARGET_DIR/schema/`.
  - In our temporary test sandbox, a target directory with pre-existing `schema/.gitkeep` received `UAS_FLIGHT_SAFETY.sysml` and subpackages without creating nested `schema/schema/` directories.

### 1.2 `scripts/install_pipeline.sh` Downstream README Scaffolding (R2)
- Trigger condition (lines 473–476):
  ```bash
  if [ ! -f "$TARGET_DIR/README.md" ] || \
     grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md" || \
     ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md" || \
     ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md"; then
  ```
  Now triggers if downstream `README.md` lacks `Customer Project Onboarding` or `.tmp-pipeline`.
- 4-Tier `$DOMAIN_REMOTE_URL` Resolution (lines 594–617):
  Resolves URL through:
  1. Tier A: Target repo remote `$REMOTE_URL` (if not `DEAP01-spec-core`).
  2. Tier B: Detected provider URL `${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${DETECTED_PROJECT}.git` (if project != `DEAP01-spec-core`).
  3. Tier B2: Project name derived URL `${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${CLEAN_NAME}.git`.
  4. Tier C: If installer root is downstream (`[ ! -e "$INSTALLER_ROOT/.pipeline/upstream" ]`), origin remote from `$INSTALLER_ROOT`.
  5. Tier D: Concrete provider fallback `${GITLAB_URL}/...` or `https://github.com/${GITHUB_ORG:-your-org}/${CLEAN_NAME}.git`.
- Downstream Section 3 scaffolding (lines 649–668):
  Scaffolds `## 3. Customer Project Onboarding & Agent Initialization Sequence` containing `### 3.1 Turnkey Customer Project Onboarding` with the single copy-pasteable command:
  ```bash
  git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
  ```
  and the in-place update command:
  ```bash
  bash scripts/install_pipeline.sh .
  ```
  No sibling paths (`../...`) or references to `DEAP01-spec-core` appear in downstream scaffolding.

### 1.3 `README.md` Preservation & Section 9 Integrity
- Lines 19–80 & 206–280 of `README.md`: Cleanly articulate Tier 1 (`DEAP01-spec-core` -> `DEAP-*`) and Tier 2 (`DEAP-*` -> `uav-*`).
- Section 9 (`## 9. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows`, line 690) remains completely intact with all prompts, subagent topologies, and verification instructions preserved.
- Code blocks in `README.md` were scanned with regex and `bash -n`:
  - 17 bash/sh blocks found.
  - Zero unescaped parentheses in comments.
  - Zero unquoted angle brackets (`<...>`).
  - All 17 blocks pass `bash -n` validation with exit code 0.

### 1.4 Verification Suite Execution
- Direct execution of `python3 scripts/verify_downstream_baseline.py --no-domain`:
  - Output: Exit code 0.
  - Checks 10 through 30 all reported `Success`.
  - Check 14 verified presence of `README.md`, `Operator Prompt Catalog`, agent entrypoints, and `rules/sysml-ssot-completeness.md`.

---

## 2. Logic Chain

1. **Premise 1 (Schema Copying):** Under POSIX and BSD `cp`, executing `cp -RP src/. dst/` when `dst/` exists copies directory contents into `dst/` rather than nesting `dst/src/`. Observation 1.1 confirms this syntax replaced the faulty `[ ! -e "$TARGET_DIR/schema" ]` guard.
2. **Premise 2 (Zero Sibling Dependency):** Customer onboarding requires single-command execution in an isolated directory without assuming local checkout of upstream or domain repositories. Observation 1.2 confirms `git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline` operates strictly inside the project root and self-cleans.
3. **Premise 3 (Remote URL Parameterization):** Parameterizing `DOMAIN_REMOTE_URL` with a 4-tier fallback ensures downstream templates point to their own repository rather than defaulting to `DEAP01-spec-core`. Observation 1.2 and empirical sandbox testing confirm that downstream READMEs embed the domain template URL.
4. **Premise 4 (Documentation & Gate Conformance):** Section 9 must remain intact to pass Check 14 of `verify_downstream_baseline.py`, and all bash code blocks must contain valid syntax without unescaped comment parentheses or unquoted angle brackets. Observations 1.3 and 1.4 confirm both conditions hold.
5. **Deduction:** The changes satisfy requirements R1, R2, and R3 without introducing regressions, breaking syntax, or violating schema-driven compiler invariants.

---

## 3. Adversarial Challenges & Stress Testing

### 3.1 Challenge 1: BSD `cp` Behavior with Pre-Existing Nested Directories
- **Assumption:** `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` handles existing subdirectories properly on macOS (BSD `cp`).
- **Attack Scenario:** Target directory contains `schema/subpkg/` with an existing file, and source contains `schema/subpkg/` with a new model.
- **Stress Test:** Executed Python subprocess test on macOS BSD `cp`.
- **Result:** Contents of `subpkg` in source merged cleanly into target `schema/subpkg/` without errors or directory duplication (`['subpkg', '.gitkeep', 'model.sysml']`). **PASS**.

### 3.2 Challenge 2: Tier C Remote Fallback in Staged Customer Repo
- **Assumption:** When a customer repo is initialized without a git remote, Tier C correctly queries the domain template's remote from `$INSTALLER_ROOT` (`.tmp-pipeline`).
- **Attack Scenario:** Customer runs installer via `.tmp-pipeline` from an uncommitted/no-remote customer directory.
- **Stress Test:** Executed sandbox test where customer repo has no remotes and `.tmp-pipeline` is a clone of a domain template.
- **Result:** Customer `README.md` correctly received `git clone https://github.com/acme/DEAP-domain.git ./.tmp-pipeline...` from Tier C resolution. **PASS**.

### 3.3 Challenge 3: In-Place Execution Refusal on `DEAP01-spec-core`
- **Assumption:** The installer refuses to overwrite or scaffold downstream README on `DEAP01-spec-core` itself.
- **Attack Scenario:** Maintainer accidentally runs `bash scripts/install_pipeline.sh .` inside `DEAP01-spec-core`.
- **Result:** Lines 163–166 check `[ -e "$INSTALLER_ROOT/.pipeline/upstream" ]` and exit with code 1: `REFUSING: target is the pipeline repository itself, not a downstream project.` Upstream `DEAP01-spec-core/README.md` is protected from being overwritten. **PASS**.

---

## 4. Integrity Audit

- **Hardcoded test results or expected outputs:** None found. `DOMAIN_REMOTE_URL` uses dynamic detection and generic fallbacks (`${GITHUB_ORG:-your-org}`).
- **Dummy or facade implementations:** None found. Directory creation and copying are genuine system calls.
- **Shortcuts or task bypasses:** None found.
- **Fabricated verification outputs:** All commands and sandbox tests were run directly in this environment, with raw command outputs recorded.
- **Self-certifying work without independent verification:** Reviewer independently reproduced and verified all claims.

---

## 5. Caveats

1. **Air-Gapped Custom GitLab Server Ports:**
   Tier B remote URL construction assumes standard URL port formatting. Non-standard ports (e.g. `https://gitlab.corp:8443`) are preserved if present in `$REMOTE_URL`, but fallback tier D uses `${GITLAB_URL}`. This is standard across all pipeline scripts.
2. **Pre-Existing Custom README in Customer Application Workspaces:**
   If an existing customer application workspace already has `Customer Project Onboarding` or `.tmp-pipeline` in its `README.md`, `scripts/install_pipeline.sh` preserves that `README.md` without overwriting it. This is intended behavior to respect downstream customizations.

---

## 6. Conclusion & Verdict

**Verdict: APPROVE**

The implementations of Work Packages 1, 2, and 3 in `README.md` and `scripts/install_pipeline.sh` are correct, robust, syntactically clean, and fully meet all acceptance criteria:
1. Two-tier architecture is documented clearly with strict separation between compiler maintenance and customer application onboarding.
2. Schema copying reliably copies domain models into pre-existing target `schema/` directories without directory nesting.
3. Downstream README scaffolding embeds the domain template's remote URL in a single, self-contained onboarding command with zero external sibling path dependencies.
4. Section 9 Operator Prompt Catalog is preserved intact.
5. All code blocks in `README.md` contain valid shell syntax with zero unescaped comment parentheses and zero unquoted angle brackets.
6. `python3 scripts/verify_downstream_baseline.py --no-domain` passes all checks cleanly with exit code 0.

---

## 7. Verification Method

To independently verify:
```bash
# 1. Baseline Conformance Gate
python3 scripts/verify_downstream_baseline.py --no-domain

# 2. Bash Script Syntax Check
bash -n scripts/install_pipeline.sh

# 3. Git Diff Scope Verification
git status
git diff origin/main scripts/install_pipeline.sh
git diff origin/main README.md
```
