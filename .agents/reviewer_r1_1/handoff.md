# Review Report & Handoff: Two-Tier Installation Architecture & Scaffolding Alignment

**Agent:** Reviewer 1 (`teamwork_preview_reviewer`, role: reviewer, critic)  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_r1_1`  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Primary Commercial Toolchain Integration Context:** `MATLAB / Simulink / Stateflow / Embedded Coder`  
**Target Work Packages Reviewed:** WP1 (R1), WP2 (R2), WP3 (R3)  
**Files Examined:** `README.md`, `scripts/install_pipeline.sh`  
**Date:** 2026-09-21  

---

## Review Summary

**Verdict:** **APPROVE**  
**Integrity Audit:** PASS (Zero integrity violations, zero hardcoded shortcuts, zero dummy facades, zero unverified claims).  
**Overall Risk Assessment:** LOW.

---

## 1. Observation

Direct examination and empirical testing of the codebase, diffs, and execution outputs revealed:

### 1.1 Scope of Changes
`git status --short` confirms only two functional files were modified:
- `README.md` (WP1 / R1)
- `scripts/install_pipeline.sh` (WP2 / R2, WP3 / R3)

### 1.2 Two-Tier Architecture Alignment in `README.md`
- **Section 1.2 (lines 19–68):** Introduces a clean ASCII architecture diagram delineating:
  - **Tier 1:** Upstream Specification Core Compiler (`DEAP01-spec-core`, `UPSTREAM_SPEC_CORE_COMPILER`)
  - **Tier 1:** Domain Distribution Templates (`DEAP-*`, `DOMAIN_DISTRIBUTION_TEMPLATE`)
  - **Tier 2:** Customer Application Workspaces (`uav-*`, `DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`)
- **Sections 5.2 & 5.3 (lines 209–271):**
  - Section 5.2 is explicitly titled: `Tier 1: Compiler Maintainer Propagation Guide (Upstream Compiler -> Domain Templates)` and documents maintainer workflows (sibling propagation, remote propagation, and in-place updates).
  - Section 5.3 is explicitly titled: `Tier 2: Customer Project Onboarding Guide (Domain Template -> Customer Workspace)` and documents customer onboarding.
  - Line 245 includes an explicit architectural rule:
    > "Critical Architecture Rule: Customer projects MUST onboard from the corresponding Domain Distribution Template (`DEAP-*`), NOT from `DEAP01-spec-core`. Onboarding from the domain template seeds your project with the required domain SysML v2 schemas, safety models, and platform profiles."
  - Lines 251–263 provide the single-command self-contained onboarding command:
    ```bash
    git clone "$DOMAIN_REMOTE_URL" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
    ```
    with zero external sibling path dependencies (`../...`).

### 1.3 Pure Valid Shell Syntax in `README.md`
- Independent execution of `.agents/reviewer_r1_1/test_readme_blocks.py` against all code blocks in `README.md` found:
  - Unescaped parentheses in shell comments: **0**
  - Unquoted angle-bracket placeholders in shell blocks: **0**
- Independent extraction and syntax testing via `bash -n` on all 17 bash/sh code fences across `README.md` (`.agents/reviewer_r1_1/test_bash_blocks_syntax.py`) resulted in:
  - **17 / 17 passed with exit code 0, 0 syntax errors**.

### 1.4 Robust Schema Copying in `scripts/install_pipeline.sh`
- Lines 283–286 of `scripts/install_pipeline.sh` state:
  ```bash
  mkdir -p "$TARGET_DIR/schema"
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
  fi
  ```
  This replaces the previous condition `if [ ! -e "$TARGET_DIR/schema" ]`, which skipped schema copying whenever `$TARGET_DIR/schema` already existed.

### 1.5 Parameterized Domain Remote URL Resolution & Scaffolding
- Lines 473–476 of `scripts/install_pipeline.sh` trigger downstream README scaffolding if missing, boilerplate, or if missing customer onboarding:
  ```bash
  if [ ! -f "$TARGET_DIR/README.md" ] || \
     grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md" || \
     ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md" || \
     ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md"; then
  ```
- Lines 594–617 implement a 4-tier resolution hierarchy for `$DOMAIN_REMOTE_URL`:
  - **Tier A:** `$REMOTE_URL` if present and does not point to `DEAP01-spec-core`.
  - **Tier B:** HTTPS URL `${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${DETECTED_PROJECT}.git` if project is not `DEAP01-spec-core`.
  - **Tier B2:** `${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${CLEAN_NAME}.git` if project name is detected.
  - **Tier C:** Query `origin` remote from `$INSTALLER_ROOT` if installer root is a domain template (`[ ! -e "$INSTALLER_ROOT/.pipeline/upstream" ]`).
  - **Tier D:** Concrete fallback URL using provider and clean directory name (`https://github.com/${GITHUB_ORG:-your-org}/${CLEAN_NAME}.git` or `${GITLAB_URL:-https://gitlab.com}/${GITLAB_GROUP:-your-group}/${CLEAN_NAME}.git`).
- Lines 649–668 scaffold Section 3 with the turnkey onboarding command embedding `${DOMAIN_REMOTE_URL}` and the in-place update command.

### 1.6 Verification Commands Output
- `bash -n scripts/install_pipeline.sh`: Exited with code 0.
- `python3 scripts/verify_downstream_baseline.py --no-domain`: Exited with code 0. All checks (Checks 10–30) verified successfully.
- Independent multi-tier end-to-end sandbox verification (`.agents/reviewer_r1_1/test_e2e_verification.py`):
  - Test 1 (Tier 1 Domain Template Installation & Remote URL Embedding): Passed.
  - Test 2 (Add Domain Models in Domain Template): Passed.
  - Test 3 (Tier 2 Customer Onboarding with Pre-Existing `schema/` containing `.gitkeep`): Passed.
  - Test 4 (Tier 2 Customer Onboarding with Existing Customer Schema Files): Passed.
  - Test 5 (Domain README Shell Syntax, Comment Parens, and Angle Brackets): Passed.

---

## 2. Logic Chain

1. **Premise 1 (R1 Alignment):** A clean two-tier architecture requires distinct workflows: Tier 1 maintainers propagate compiler capabilities into domain distribution templates (`DEAP-*`), while Tier 2 end-users onboard from domain templates into customer workspaces (`uav-*`). Conflating these leads customer workspaces to clone `DEAP01-spec-core`, which lacks domain models and fails down-pipeline engineering.
2. **Observation Reference:** As observed in Section 1.2, `README.md` clearly demarcates Tier 1 from Tier 2 in Section 1.2, Section 4, Section 5.2, and Section 5.3, with a clear warning that customer workspaces must onboard from `DEAP-*` rather than `DEAP01-spec-core`.
3. **Premise 2 (R2 Parameterization):** Downstream domain templates must provide self-contained onboarding instructions pointing to their own git remote URL, eliminating `../...` sibling path assumptions.
4. **Observation Reference:** As observed in Section 1.5, `scripts/install_pipeline.sh` automatically detects the domain repository's remote URL via a 4-tier resolution ladder and embeds it directly into the generated `README.md` Section 3.1. In sandbox testing (Observation 1.6), the generated command was `git clone https://github.com/defense-contractor/DEAP-uas-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`, with zero reference to `DEAP01-spec-core`.
5. **Premise 3 (R3 Robust Schema Copy):** Downstream customer workspaces often begin with an empty `schema/` directory containing `.gitkeep`. An installer that checks `if [ ! -e "$TARGET_DIR/schema" ]` will skip schema installation entirely.
6. **Observation Reference:** As observed in Section 1.4 and verified in Observation 1.6 (Tests 3 & 4), `mkdir -p "$TARGET_DIR/schema"` followed by `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` copies all schema files and subdirectories into `$TARGET_DIR/schema/` without creating nested `schema/schema/` directories and without erasing pre-existing customer schema files.
7. **Premise 4 (Syntax Integrity):** Markdown shell code blocks must not contain unescaped parentheses in comments (which cause shell parser issues in automated tooling) or unquoted angle-bracket placeholders.
8. **Observation Reference:** As observed in Section 1.3, both `README.md` and scaffolded downstream `README.md` files were empirically verified to contain 0 unescaped parentheses in comments, 0 unquoted angle-bracket placeholders, and 100% pass `bash -n`.
9. **Deduction:** The changes satisfy all requirements (R1, R2, R3) and acceptance criteria specified in `ORIGINAL_REQUEST.md` and `implementation_plan.md` without regression.

---

## 3. Caveats

- **Existing Downstream Customized READMEs:** In `scripts/install_pipeline.sh`, if a downstream repository already possesses a custom `README.md` that contains both the `Operator Prompt Catalog` and `Customer Project Onboarding` (or `.tmp-pipeline`), the script intentionally preserves that customized file without overwriting it.
- **In-Place Upstream Execution:** Per lines 163–167 of `scripts/install_pipeline.sh`, the script continues to enforce the safety invariant that it cannot be executed with `$TARGET_DIR` set to `DEAP01-spec-core` itself.

---

## 4. Conclusion

The implementation across Work Packages 1, 2, and 3 is verified, correct, robust, and complete:
- **Verdict:** **APPROVE**
- `README.md` cleanly establishes the two-tier architecture boundary without tier conflation.
- `scripts/install_pipeline.sh` reliably copies domain schemas into target directories even when `schema/` pre-exists, avoiding nesting.
- Downstream README scaffolding dynamically embeds the domain remote URL and emits pure valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle brackets.
- Baseline verification script `python3 scripts/verify_downstream_baseline.py --no-domain` passes with exit code 0.

---

## 5. Verification Method

To independently reproduce and verify this assessment:

1. **Downstream Baseline Gate:**
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected Output:* Exits with code 0; all Checks 10–30 succeed.

2. **README Code Block Syntax & Shell Validation:**
   ```bash
   python3 /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_r1_1/test_readme_blocks.py
   python3 /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_r1_1/test_bash_blocks_syntax.py
   ```
   *Expected Output:* 0 violations, all 17 bash blocks pass `bash -n`.

3. **Multi-Tier End-to-End Sandbox Test:**
   ```bash
   python3 /Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_r1_1/test_e2e_verification.py
   ```
   *Expected Output:* All 5 tests pass; prints `ALL VERIFICATION TESTS COMPLETED SUCCESSFULLY!`.

4. **Installer Script Shell Syntax Check:**
   ```bash
   bash -n scripts/install_pipeline.sh
   ```
   *Expected Output:* Exits with code 0.

5. **Invalidation Conditions:**
   - Any non-zero exit code from `python3 scripts/verify_downstream_baseline.py --no-domain`.
   - Failure of `install_pipeline.sh` to copy schemas when `schema/` pre-exists with `.gitkeep`.
   - Creation of nested `schema/schema/` in target workspaces.
   - Any occurrence of `DEAP01-spec-core` in generated downstream customer onboarding instructions.
   - Unescaped parentheses in comments or unquoted `<...>` in markdown bash blocks.

---

## Verified Claims Matrix

| Claim | Source | Verification Method | Status |
|---|---|---|---|
| Clean two-tier architecture documentation without conflating Tier 1 maintainers with Tier 2 customer onboarding | `README.md` | Direct path read of Sections 1.2, 4, 5.2, 5.3 | **PASS** |
| Zero unescaped parens in comments in `README.md` | `README.md` | AST regex audit via `test_readme_blocks.py` | **PASS** |
| Zero unquoted angle brackets in `README.md` shell blocks | `README.md` | Regex parser via `test_readme_blocks.py` | **PASS** |
| Pure valid shell syntax in all `README.md` bash code fences | `README.md` | Automated extraction & `bash -n` on all 17 blocks | **PASS** |
| Robust schema copy into existing `schema/` directory | `scripts/install_pipeline.sh` | End-to-end sandbox test (`test_e2e_verification.py`) | **PASS** |
| Prevention of nested `schema/schema/` directory | `scripts/install_pipeline.sh` | End-to-end sandbox test (`test_e2e_verification.py`) | **PASS** |
| Preservation of existing customer schemas during install | `scripts/install_pipeline.sh` | End-to-end sandbox test (`test_e2e_verification.py`) | **PASS** |
| Downstream README scaffolds domain remote URL (no spec-core references) | `scripts/install_pipeline.sh` | End-to-end sandbox test (`test_e2e_verification.py`) | **PASS** |
| Downstream README scaffolds turnkey customer onboarding command | `scripts/install_pipeline.sh` | End-to-end sandbox test (`test_e2e_verification.py`) | **PASS** |
| Clean baseline verification | Repository root | `python3 scripts/verify_downstream_baseline.py --no-domain` | **PASS** |

---

## Adversarial Challenge & Stress Test Report

| Scenario / Hypothesis | Stress Test | Result | Assessment |
|---|---|---|---|
| **H1: SSH Remote URL Handling:** Target domain repository uses SSH remote (`git@github.com:...`). Does it break URL parsing or inject invalid characters? | Tested with `git@github.com:defense-contractor/DEAP-uas-safety.git` in temporary sandbox. | Scaffolding parsed and emitted valid `git clone git@github.com:...` command. | **PASS (Robust)** |
| **H2: No Git Remote Configured:** Target directory has no git remotes. Does it emit broken `<...>` angle brackets? | Tested installer on uninitialized directory. | Scaffolding triggered Tier D fallback: `https://github.com/your-org/DEAP-medical-device.git`, zero `<>` angle brackets. | **PASS (Robust)** |
| **H3: Directory Name with Spaces:** Domain project name contains spaces (e.g. `UAS Safety Platform`). | Tested project name sanitization. | `CLEAN_NAME=$(echo "$DOMAIN_PROJECT_NAME" \| tr ' ' '-')` sanitized spaces to hyphens. | **PASS (Robust)** |
| **H4: Integrity Check:** Did workers commit hardcoded outputs, dummy facades, or self-certifying shortcuts? | Inspected diffs in `scripts/install_pipeline.sh` and `README.md`. | Implementation uses generic dynamic resolution ladders and standard POSIX shell commands. | **PASS (No Violations)** |
