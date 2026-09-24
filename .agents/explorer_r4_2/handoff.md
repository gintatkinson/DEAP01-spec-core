# Handoff Report: `scripts/install_pipeline.sh` README Scaffolding & Tier Role Branching

**Agent:** `explorer_r4_2`  
**Archetype:** Explorer  
**Task:** Survey and investigate `scripts/install_pipeline.sh` README scaffolding logic, role detection, and branching for domain templates vs customer projects.  
**Date:** 2026-09-21  

---

## 1. Observation

1. **Monolithic Hardcoded Scaffolding in `scripts/install_pipeline.sh`**:
   - At line 670 of `scripts/install_pipeline.sh`:
     ```markdown
     > **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`
     ```
   - At lines 701-706 of `scripts/install_pipeline.sh`:
     ```bash
     To install this domain pipeline and its engineering baseline into an end-user customer application workspace, run the following turnkey command from your customer project root directory:

     ```bash
     # Onboard customer application workspace
     git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
     ```
     ```
   - When run in a customer project (e.g. `uav-011`), `${DOMAIN_REMOTE_URL}` evaluates to the customer repo itself (e.g. `https://gitlab.com/uas-safety/uav-011.git`), generating circular instructions for `uav-011` to clone `uav-011` into `.tmp-pipeline`.

2. **README Scaffolding Trigger Condition**:
   - At lines 517-520 of `scripts/install_pipeline.sh`:
     ```bash
     if [ ! -f "$TARGET_DIR/README.md" ] || \
        grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md" || \
        ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md" || \
        ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md"; then
     ```
   - If a customer README does not contain `"Customer Project Onboarding"` or `"\.tmp-pipeline"`, this condition evaluates to true on subsequent runs and forces an overwrite.

3. **Existing Role Detection in Adjacent Tooling**:
   - In `scripts/dispatch_subagent.py` lines 179-240:
     - Detects `PARENT_DOMAIN_DISTRIBUTION_TEMPLATE`, `CHILD_DOMAIN_DISTRIBUTION_TEMPLATE`, `DOWNSTREAM_APPLICATION_WORKSPACE`, and `UPSTREAM_SPEC_CORE_COMPILER`.
     - Checks CLI arguments, environment variables, `.pipeline/lineage.json`, `README.md`, and `.pipeline/upstream` sentinel marker.
   - In `scripts/scaffold_downstream_agents.py` lines 18-21:
     - Header hardcoded to `DOWNSTREAM_CUSTOMER_PROJECT` / `DOMAIN_TEMPLATE_CHILD`.
   - In `scripts/install_pipeline.sh` lines 54-184:
     - There is currently **zero CLI parsing for `--role` or `--tier`**.

4. **Existing Test Suite Baseline**:
   - `python3 -m unittest tests/test_domain_url_synthesis.py` runs 9 tests and exits 0 in 6.6s.
   - `python3 scripts/verify_downstream_baseline.py --no-domain` passes all 30 checks cleanly.
   - In `scripts/verify_downstream_baseline.py` lines 516-524 (Check 14):
     ```python
     if not (
         "# Downstream Cyber-Physical Infrastructure Safety Project" in readme_content
         or "Operator Prompt Catalog" in readme_content
     ):
         print("ERROR: Check 14 failed...")
     ```
     Check 14 accepts either the title `# Downstream Cyber-Physical Infrastructure Safety Project` or the heading `"Operator Prompt Catalog"`.

---

## 2. Logic Chain

1. **Step 1 (Trigger Invalidation)**: From Observation 1 and 2, `scripts/install_pipeline.sh` currently has no role branching. It assumes every target directory is a domain repository capable of spawning customer workspaces, and insists that `README.md` contain `"Customer Project Onboarding"`.
2. **Step 2 (Role Identification)**: From Observation 3, downstream targets fall into two distinct tiers:
   - Tier 1 Domain Distribution Templates (`DEAP-*`, e.g. `DEAP-uas-infrastructure-safety`)
   - Tier 2 Customer Application Workspaces (`uav-*`, e.g. `uav-011`)
3. **Step 3 (Branching Mandate)**:
   - When installing into a Domain Template (`DEAP-*`): The README must declare `DOMAIN_DISTRIBUTION_TEMPLATE`, document the clean landing zone invariant, and provide the customer onboarding command `git clone <domain-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`.
   - When installing into a Customer Workspace (`uav-*`): The README must declare `DOWNSTREAM_CUSTOMER_PROJECT`, eliminate the circular clone command, and document project-specific commands: baseline verification (`verify_downstream_baseline.py --no-domain`), Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py`), and in-place tooling update (`bash scripts/install_pipeline.sh .`).
4. **Step 4 (Syntax Compliance)**: From Observation 1, all generated bash code blocks must have valid shell syntax, quote all variables/URLs (`"${DOMAIN_REMOTE_URL}"`), avoid bare `<angle-brackets>`, and avoid unescaped parentheses in comments (e.g. `# Option A: ...` instead of `# (Option A)`).
5. **Step 5 (Backward Compatibility)**: From Observation 4, Check 14 passes as long as "Operator Prompt Catalog" is present in both variants. All 9 tests in `test_domain_url_synthesis.py` continue passing because domain template flags (`--domain-name DEAP-*` and `--domain-url`) will resolve to `DOMAIN_DISTRIBUTION_TEMPLATE`.

---

## 3. Caveats

1. **Installer Scope**: This investigation is read-only and analyzed `scripts/install_pipeline.sh` and its immediate consumers. The upstream compiler's top-level `README.md` (DEAP01-spec-core/README.md) is surveyed in parallel by `explorer_r4_1`.
2. **In-Place Execution on Customer Workspace**: When running `bash scripts/install_pipeline.sh .` inside an existing customer workspace (`uav-011`), `$TARGET_DIR` equals `$INSTALLER_ROOT`. Detection relies on `DIR_BASE` (e.g. `uav-011`), git remote project name, or `--role customer-project`.
3. **External Domain Repositories**: Non-canonical customer projects that do not start with `uav-` will default to `DOWNSTREAM_CUSTOMER_PROJECT` unless `--role domain-template` or `--domain-name DEAP-*` is provided.

---

## 4. Conclusion

1. **Add `--role` CLI Option**: Add `-r, --role ROLE` to `scripts/install_pipeline.sh` accepting `domain-template` / `customer-project` (and uppercase/underscore variants).
2. **Implement Dynamic Role Resolution**:
   - `DOMAIN_DISTRIBUTION_TEMPLATE`: if `--role` is domain-template, target directory starts with `DEAP-`, git remote project starts with `DEAP-`, `--domain-name` starts with `DEAP-`, or `--domain-url` is specified.
   - `DOWNSTREAM_CUSTOMER_PROJECT`: if `--role` is customer-project, target directory starts with `uav-`, git remote project starts with `uav-`, or by fallback default.
3. **Split README Scaffolding into Distinct Branches**:
   - Role-aware regeneration check replacing lines 517-520.
   - Dedicated Section 1 & Section 3 content per role.
   - Zero circular clone commands in customer workspaces.
4. **Full details and replacement templates are documented in `report.md`**.

---

## 5. Verification Method

1. **Run Existing Domain URL Synthesis Tests**:
   ```bash
   python3 -m unittest tests/test_domain_url_synthesis.py
   ```
   *Expected: All 9 tests pass.*

2. **Run Downstream Baseline Verification**:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected: All 30 checks pass.*

3. **Verify Generated Code Block Syntax**:
   ```bash
   bash -n scripts/install_pipeline.sh
   ```
   *Expected: Clean exit code 0 without syntax errors.*

4. **New Test Suite Execution (`tests/test_readme_scaffolding.py`)**:
   Verify distinct README generation, absence of circular clone commands in `uav-*`, and presence of customer project commands.
