# Handoff Report: Milestone 2 (R2 and R3) - Distinct README Scaffolding & Dynamic Role Detection

**Agent:** `worker_m2_1`  
**Role:** Implementer / QA  
**Task:** Milestone 2 (R2 and R3) - scripts/install_pipeline.sh  
**Date:** 2026-09-21  

---

## 1. Observation

1. **Previous Defect in `scripts/install_pipeline.sh`**:
   - Lines 670-716 previously hardcoded:
     ```markdown
     > **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`
     ```
     and generated a single onboarding section instructing customer projects (e.g. `uav-011`) to clone themselves into `.tmp-pipeline`:
     ```bash
     git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
     ```
   - Lines 517-520 required `Customer Project Onboarding|\.tmp-pipeline` in existing READMEs, which would trigger an unnecessary overwrite if a customer project README did not include circular self-clone commands.
   - CLI options had no `--role` / `-r` parameter, preventing explicit override of repository role.

2. **Implemented Implementation**:
   - Added `-r, --role ROLE` CLI option parsing and validation for domain templates and customer workspaces.
   - Added deterministic auto-detection when `--role` is omitted:
     - `DEAP-*` directory name, remote URL containing `DEAP-`, `--domain-name DEAP-*`, or non-empty `--domain-url` -> `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - Otherwise (or if `uav-*`) -> `DOWNSTREAM_CUSTOMER_PROJECT`.
   - Replaced trigger condition at lines 517-520 with role-aware `SHOULD_SCAFFOLD_README` logic.
   - Distinct README scaffolding:
     - `DOMAIN_DISTRIBUTION_TEMPLATE`: Declares `DOMAIN_DISTRIBUTION_TEMPLATE`, enforces clean landing zone invariant, provides single-line customer onboarding clone command, and in-place domain template update command.
     - `DOWNSTREAM_CUSTOMER_PROJECT`: Declares `DOWNSTREAM_CUSTOMER_PROJECT`, documents project workspace scope, eliminates circular clone command, documents downstream baseline verification (`python3 scripts/verify_downstream_baseline.py --no-domain`), Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py` / `compile_sysml.py`), and in-place tooling update (`bash scripts/install_pipeline.sh .`).
     - Both variants preserve Section 4 Operator Prompt Catalog and Section 5 Quality Gates.
   - Clean shell code blocks with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

---

## 2. Logic Chain

1. **Role Separation**: Upstream spec compiler (`DEAP01-spec-core`), domain templates (`DEAP-*`), and customer workspaces (`uav-*`) serve distinct architectural purposes. Domain templates distribute schemas and must instruct customers how to onboard. Customer workspaces implement application software and must execute verification and ingestion workflows without circular instructions to clone themselves.
2. **Deterministic Role Invariant**: By prioritizing explicit `--role` CLI flags, then matching `DEAP-*` vs `uav-*` patterns in directory names, git remotes, and domain flags, the installer deterministically chooses the correct scaffolding path without user intervention while allowing manual override.
3. **Idempotence and Upgrade**: Evaluating `SHOULD_SCAFFOLD_README` per role guarantees that compliant READMEs are not re-scaffolded on repetitive installer runs (`bash scripts/install_pipeline.sh .`), while legacy READMEs containing circular clone instructions are safely upgraded to the new clean customer template.
4. **Toolchain & Standards Conformance**: Both scaffolding branches declare MATLAB / Simulink / Stateflow / Embedded Coder as the primary commercial toolchain context and preserve Section 4 (Operator Prompt Catalog) and Section 5 (Verification & Quality Gates), ensuring Check 14 in `scripts/verify_downstream_baseline.py` passes.

---

## 3. Caveats

- `scripts/install_pipeline.sh` was modified under exclusive write ownership. No other repository source or test files were modified.
- Milestone 3 will provide any additional automated unit tests in `tests/test_readme_scaffolding.py`. All hermetic tests executed during this milestone passed without issue.

---

## 4. Conclusion

Milestone 2 (R2 and R3) is complete, robust, and verified:
- Dynamic repository role detection implemented with `-r, --role` and auto-detection.
- Distinct README templates scaffolded for `DOMAIN_DISTRIBUTION_TEMPLATE` and `DOWNSTREAM_CUSTOMER_PROJECT`.
- Circular self-cloning instructions in customer projects are completely eliminated.
- Shell syntax hygiene strictly enforced across all generated code fences.
- All test suites and downstream baseline checks pass cleanly with zero regressions.

---

## 5. Verification Method

1. **Syntax Check**:
   ```bash
   bash -n scripts/install_pipeline.sh
   ```
   *Result: Exit 0, syntax valid.*

2. **Existing Test Discovery**:
   ```bash
   python3 -m unittest discover tests
   ```
   *Result: Ran 23 tests, all OK.*

3. **Downstream Baseline Verification**:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Result: All 30 checks verified successfully, exit 0.*

4. **Hermetic Multi-Role Test Suite**:
   Executed hermetically in temporary directories outside the workspace:
   - `--role domain-template` CLI flag: Verified `DOMAIN_DISTRIBUTION_TEMPLATE`, clean landing zones, customer clone command.
   - `--role customer-project` CLI flag: Verified `DOWNSTREAM_CUSTOMER_PROJECT`, no circular clone, project verification & ingestion commands present.
   - Auto-detection on `DEAP-*` directory: Verified `DOMAIN_DISTRIBUTION_TEMPLATE`.
   - Auto-detection on `uav-*` directory: Verified `DOWNSTREAM_CUSTOMER_PROJECT`.
   - Idempotence: Verified second execution does not overwrite compliant READMEs.
   - Upgradability: Verified legacy circular READMEs are regenerated and upgraded.
   - Shell hygiene: Verified all ```bash blocks pass `bash -n` and have zero unescaped parens in comments or unquoted angle brackets.
