# Handoff Report: Milestone 2 Review - Distinct README Scaffolding & Dynamic Role Detection

**Agent:** `reviewer_m2_1`  
**Roles:** Reviewer, Adversarial Critic  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/reviewer_m2_1`  
**Date:** 2026-09-21  

---

## 1. Observation

1. **Target File Under Review**: `scripts/install_pipeline.sh` (lines 10-79, 213-228, 323-345, 578-898).
2. **CLI Option Parsing (`-r, --role ROLE`)**:
   - Added `CLI_ROLE=""` variable definition (line 15).
   - Documented option in `show_help()`: `-r, --role ROLE Target repository role: 'domain-template' or 'customer-project' (auto-detected if omitted)`.
   - Option parsing handles `-r|--role`, `--role=*`, and `-r=*`. Aborts with `exit 1` if role argument is missing or begins with `-`.
   - Normalization and validation at lines 214-227:
     ```bash
     if [ -n "$CLI_ROLE" ]; then
       case "$(echo "$CLI_ROLE" | tr '[:upper:]' '[:lower:]' | tr '-' '_')" in
         domain_template|domain|domain_distribution_template)
           TARGET_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"
           ;;
         customer_project|customer|downstream_customer_project|downstream_application_workspace|workspace)
           TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
           ;;
         *)
           echo "Error: Invalid --role '$CLI_ROLE'. Valid values: 'domain-template', 'customer-project', 'DOMAIN_DISTRIBUTION_TEMPLATE', 'DOWNSTREAM_CUSTOMER_PROJECT'." >&2
           exit 1
           ;;
       esac
     fi
     ```
3. **Dynamic Repository Role Auto-Detection** (lines 324-343):
   - When `$TARGET_ROLE` is empty, checks:
     - `[[ "$DIR_BASE" == DEAP-* ]]` or `[[ "$REMOTE_URL" == *DEAP-* ]]` or `[[ "$DETECTED_PROJECT" == DEAP-* ]]` or `[[ "$DOMAIN_NAME" == DEAP-* ]]` or `[ -n "$DOMAIN_URL" ]` -> `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - `[[ "$DIR_BASE" == uav-* ]]` or `[[ "$DETECTED_PROJECT" == uav-* ]]` -> `DOWNSTREAM_CUSTOMER_PROJECT`.
     - `.pipeline/lineage.json` metadata inspection fallback.
     - Default fallback -> `DOWNSTREAM_CUSTOMER_PROJECT`.
4. **Distinct Scaffolding by Role** (lines 747-898):
   - `DOMAIN_DISTRIBUTION_TEMPLATE`:
     - Sets header `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\``.
     - Section 1.1 documents Clean Landing Zone Invariant.
     - Section 3.1 documents customer onboarding clone command:
       ```bash
       git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
       ```
     - Section 3.2 documents in-place domain template tooling update (`bash scripts/install_pipeline.sh .`).
   - `DOWNSTREAM_CUSTOMER_PROJECT`:
     - Sets header `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``.
     - Section 1.1 documents Customer Application Workspace Scope.
     - Section 3.1 documents baseline verification (`python3 scripts/verify_downstream_baseline.py --no-domain`) and Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py` and `compile_sysml.py --compile`).
     - Section 3.2 documents in-place pipeline tooling update (`bash scripts/install_pipeline.sh .`).
     - Zero circular clone commands (`git clone.*\.tmp-pipeline` completely absent).
   - Both variants preserve Section 4 Operator Prompt Catalog and Section 5 Quality Gates.
5. **Regeneration Trigger Condition** (lines 578-597):
   - Sets `SHOULD_SCAFFOLD_README=true` if README is missing, contains GitLab boilerplate, or lacks Operator Prompt Catalog.
   - For `DOMAIN_DISTRIBUTION_TEMPLATE`, triggers re-scaffolding if missing `Customer Project Onboarding` or `DOMAIN_DISTRIBUTION_TEMPLATE`.
   - For `DOWNSTREAM_CUSTOMER_PROJECT`, triggers re-scaffolding if containing circular `git clone.*\.tmp-pipeline`, or missing `DOWNSTREAM_CUSTOMER_PROJECT` or `Project Lifecycle & Tooling Maintenance`.
   - Idempotently preserves compliant READMEs without overwriting them.
6. **Tool Commands and Results**:
   - `bash -n scripts/install_pipeline.sh`: Exited with code 0.
   - `python3 -m unittest discover tests`: Ran 23 tests in 6.704s, OK (exit 0).
   - `python3 scripts/verify_downstream_baseline.py --no-domain`: All 30 checks verified successfully (exit 0).
   - Independent 9-test adversarial test harness executed in `/tmp`: All 9 tests passed.

---

## 2. Logic Chain

1. **Premise 1**: Upstream compiler, domain distribution templates, and downstream customer workspaces serve distinct architectural tiers. Scaffolding in domain templates must guide customers to clone and install, whereas customer workspaces must guide engineers to verify, ingest, and maintain in-place without circular self-cloning.
2. **Premise 2**: Observation 2 and Observation 3 confirm that role detection is implemented deterministically with explicit CLI override priority (`-r, --role`), followed by clean heuristic matching on directory name prefixes (`DEAP-*` vs `uav-*`), git remotes, and lineage metadata.
3. **Premise 3**: Observation 4 confirms that the two generated README variants are strictly tailored to their respective roles, eliminating circular self-cloning instructions in customer projects while preserving necessary prompt catalogs and quality gates.
4. **Premise 4**: Observation 5 confirms that the regeneration condition preserves compliant user-customized READMEs across routine in-place updates, while ensuring legacy circular READMEs are cleanly upgraded.
5. **Premise 5**: Observation 6 demonstrates that all existing unit tests and downstream baseline validation gates pass without regression, and all adversarial edge cases pass.
6. **Conclusion**: The Milestone 2 implementation in `scripts/install_pipeline.sh` is complete, correct, and approved.

---

## 3. Caveats

No caveats. All aspects of the Milestone 2 implementation were independently verified both within the existing workspace and via isolated temporary test environments.

---

## 4. Conclusion

**Verdict: APPROVE**.
The changes to `scripts/install_pipeline.sh` satisfy all Milestone 2 requirements (R2 and R3). No integrity violations or logic flaws were identified.

---

## 5. Verification Method

To independently reproduce and verify this assessment:

1. **Syntax Check**:
   ```bash
   bash -n scripts/install_pipeline.sh
   ```
   *Expected: Exit code 0, no output.*

2. **Existing Test Suite**:
   ```bash
   python3 -m unittest discover tests
   ```
   *Expected: Ran 23 tests, all OK (exit code 0).*

3. **Downstream Baseline Gate**:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected: All 30 checks verified successfully, exit code 0.*

4. **Multi-Role Scaffolding Verification**:
   Execute the isolated Python test harness in a temporary directory:
   ```bash
   python3 -c '
   import tempfile, subprocess, os, re
   INSTALLER = "/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh"
   with tempfile.TemporaryDirectory() as tmpdir:
       d = os.path.join(tmpdir, "uav-011")
       os.makedirs(d)
       subprocess.run(["bash", INSTALLER, "--role", "customer-project", d], check=True)
       with open(os.path.join(d, "README.md")) as f:
           c = f.read()
       assert "> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`" in c
       assert not re.search(r"git clone.*\.tmp-pipeline", c)
       assert "verify_downstream_baseline.py" in c
       print("Customer project verification successful.")
   '
   ```
