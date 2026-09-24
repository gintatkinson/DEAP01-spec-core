# Handoff Report: Challenger M2-1 (`scripts/install_pipeline.sh`)

## 1. Observation

1. **CLI Role Parsing & Validation**:
   - `scripts/install_pipeline.sh:30`: `-r, --role ROLE` option documented in `--help`.
   - `scripts/install_pipeline.sh:64-79`: CLI flags `-r`, `--role`, `--role=*`, and `-r=*` are parsed. If `$2` is empty or starts with `-`, script emits verbatim:
     `Error: $1 requires a role argument ('domain-template' or 'customer-project').` and exits with code 1.
   - `scripts/install_pipeline.sh:213-227`: `CLI_ROLE` is normalized via `tr '[:upper:]' '[:lower:]' | tr '-' '_'`. Unrecognized role strings trigger:
     `Error: Invalid --role '$CLI_ROLE'. Valid values: 'domain-template', 'customer-project', 'DOMAIN_DISTRIBUTION_TEMPLATE', 'DOWNSTREAM_CUSTOMER_PROJECT'.` and exit with code 1.

2. **Dynamic Role Detection & Scaffolding**:
   - `scripts/install_pipeline.sh:324-343`: If `TARGET_ROLE` is unset, directory base matching `DEAP-*` or remote URL containing `DEAP-*` resolves to `DOMAIN_DISTRIBUTION_TEMPLATE`.
   - Directory base matching `uav-*` or default fallback resolves to `DOWNSTREAM_CUSTOMER_PROJECT`.
   - `scripts/install_pipeline.sh:586-597`: README generation gate evaluates `TARGET_ROLE`.
     For `DOWNSTREAM_CUSTOMER_PROJECT`, if `git clone.*\.tmp-pipeline` is present in an existing README, `SHOULD_SCAFFOLD_README` is set to `true` to overwrite legacy circular clone instructions.

3. **Distinct README Scaffolding Content**:
   - Lines 746-816: `DOMAIN_DISTRIBUTION_TEMPLATE` README generates:
     - Badge: `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\``
     - Section 1.1: `### 1.1 Clean Landing Zone Invariant`
     - Section 3.1: Turnkey Customer Onboarding command:
       `git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
     - Section 3.1: In-place update: `bash scripts/install_pipeline.sh .`
   - Lines 817-898: `DOWNSTREAM_CUSTOMER_PROJECT` README generates:
     - Badge: `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``
     - Section 1.1: `### 1.1 Customer Application Workspace Scope`
     - Section 3.1: Baseline Verification (`python3 scripts/verify_downstream_baseline.py --no-domain`) and Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py`)
     - Section 3.2: In-place pipeline tooling update: `bash scripts/install_pipeline.sh .`
     - Verbatim scan: contains **ZERO** occurrences of `git clone ... uav-011` and **ZERO** occurrences of `.tmp-pipeline`.

4. **Empirical Stress Test Execution**:
   - Hermetic test script `/tmp/test_challenger_install.py` executed across 18 automated test cases in temporary directories (`tempfile.TemporaryDirectory()`).
   - Command: `python3 /tmp/test_challenger_install.py`
   - Result: `Total Tests Run: 18, Passed: 18, Failed: 0. Exit code 0.`
   - Code fence syntax check: `bash -n` executed against all shell code fences across all generated READMEs -> 0 syntax errors.
   - Comment check: 0 unescaped parentheses.
   - Angle-bracket check: 0 unquoted angle-bracket placeholders.
   - Conformance check: `python3 scripts/verify_downstream_baseline.py --no-domain` exited 0.
   - Unit test suite: `python3 -m unittest discover -s tests -p "test_*.py"` ran 23 tests -> `OK` (0 failures).

---

## 2. Logic Chain

1. **Role Resolution Logic**:
   - Observation 1 demonstrates that explicit CLI flags `--role` / `-r` are validated first and take precedence over all directory name heuristics (verified in Tests 3, 4, 8.1, 8.2).
   - When no CLI flag is provided, Observation 2 establishes deterministic auto-detection based on `DIR_BASE` prefix (`DEAP-*` vs `uav-*`), verified in Tests 1 and 2.
   - Observation 1 establishes that invalid roles cleanly abort with exit code 1 and do not leave partially scaffolded artifacts (verified in Test 5).

2. **README Generation & Circular Clone Prevention**:
   - From Observation 3, the customer project README generation path completely omits the `git clone ... .tmp-pipeline` onboarding block and replaces it with `bash scripts/install_pipeline.sh .`.
   - From Observation 2 and Test 9, existing customer repositories that previously had circular clone commands are detected via `grep -qE "git clone.*\.tmp-pipeline"` and automatically regenerated without circular commands.
   - From Observation 3, domain distribution templates strictly receive the Clean Landing Zone Invariant and the onboarding clone command using the resolved domain repository remote URL.

3. **Shell Script Robustness**:
   - From Observation 4, all generated shell code fences parse without error under `bash -n`.
   - Zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders prevent shell execution redirection traps and parser failures.

---

## 3. Caveats

- **Network-Isolated Git Remote Operations**: In automated test runs, `git clone` from live remote network hosts was not executed over the public internet to ensure hermetic sandbox execution; instead, local repository structures and synthetic remote URLs were tested to verify URL resolution heuristics.
- **Case-Sensitivity of Directory Heuristics**: Auto-detection matches `DEAP-*` and `uav-*` using standard Bash case-sensitive globbing. Custom or lowercase domain template directory names (e.g. `deap-test`) require passing explicit `--role domain-template`, which is explicitly supported and documented.

---

## 4. Conclusion

**Verdict: APPROVE.**

The implementation of dynamic role detection and README scaffolding in `scripts/install_pipeline.sh` satisfies all requirements of R2 and R3 in `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md`:
1. Distinguishes `DOMAIN_DISTRIBUTION_TEMPLATE` and `DOWNSTREAM_CUSTOMER_PROJECT`.
2. Generates customer onboarding commands and Clean Landing Zone invariants for Domain Distribution Templates.
3. Generates zero circular self-clone commands and embeds in-place tooling updates (`bash scripts/install_pipeline.sh .`) for Customer Application Workspaces.
4. Robustly handles all CLI `--role` syntax variants (`--role=...`, `-r ...`, `-r=...`) and cleanly errors on invalid roles.
5. All generated Markdown and Shell blocks pass `bash -n` syntax checks with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

---

## 5. Verification Method

To independently reproduce and verify this assessment:

1. **Execute the Hermetic Challenger Test Suite**:
   ```bash
   python3 /tmp/test_challenger_install.py
   ```
   *Expected output*: `ALL STRESS TESTS PASSED EMPIRICALLY.` (Exit code 0).

2. **Run Downstream Conformance Baseline Verification**:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected output*: All checks pass cleanly with exit code 0.

3. **Run Unit & Regression Tests**:
   ```bash
   python3 -m unittest discover -s tests -p "test_*.py"
   ```
   *Expected output*: `Ran 23 tests ... OK` (Exit code 0).

4. **Verify Manual Execution**:
   ```bash
   # Test domain template auto-detection
   rm -rf /tmp/test-dt && bash scripts/install_pipeline.sh /tmp/DEAP-uas-infrastructure-safety
   grep "DOMAIN_DISTRIBUTION_TEMPLATE" /tmp/DEAP-uas-infrastructure-safety/README.md
   grep "Clean Landing Zone Invariant" /tmp/DEAP-uas-infrastructure-safety/README.md

   # Test customer project auto-detection
   rm -rf /tmp/uav-011 && bash scripts/install_pipeline.sh /tmp/uav-011
   grep "DOWNSTREAM_CUSTOMER_PROJECT" /tmp/uav-011/README.md
   ! grep "git clone.*uav-011" /tmp/uav-011/README.md
   ! grep "\.tmp-pipeline" /tmp/uav-011/README.md

   # Test invalid role exit
   bash scripts/install_pipeline.sh --role invalid-role /tmp/test-target; echo "Exit code: $?"
   ```
