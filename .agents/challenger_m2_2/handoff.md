# Handoff Report — challenger_m2_2

**Task:** Adversarial verification of execution stability and idempotence of `scripts/install_pipeline.sh` (Milestone M2.2)  
**Agent:** Empirical Challenger (`challenger_m2_2`)  
**Verdict:** **APPROVE**  

---

## 1. Observation

Direct observations and empirical evidence gathered during test execution:

1. **Scaffolding Logic in `scripts/install_pipeline.sh`:**
   - Lines 586–597:
     ```bash
     elif [ "$TARGET_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ]; then
       if ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md" || \
          ! grep -qE "DOMAIN_DISTRIBUTION_TEMPLATE" "$TARGET_DIR/README.md"; then
         SHOULD_SCAFFOLD_README=true
       fi
     elif [ "$TARGET_ROLE" = "DOWNSTREAM_CUSTOMER_PROJECT" ]; then
       if grep -qE "git clone.*\.tmp-pipeline" "$TARGET_DIR/README.md" || \
          ! grep -qE "DOWNSTREAM_CUSTOMER_PROJECT" "$TARGET_DIR/README.md" || \
          ! grep -qE "Project Lifecycle & Tooling Maintenance" "$TARGET_DIR/README.md"; then
         SHOULD_SCAFFOLD_README=true
       fi
     fi
     ```
   - Lines 818–898:
     Generates distinct customer README with:
     - `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``
     - `## 3. Project Lifecycle & Tooling Maintenance`
     - `### 3.1 Downstream Baseline Verification & Ingestion Workflows` (`python3 scripts/verify_downstream_baseline.py --no-domain`)
     - `### 3.2 In-Place Pipeline Tooling Update` (`bash scripts/install_pipeline.sh .`)
     - Zero references to `git clone.*\.tmp-pipeline`

2. **Empirical Execution Results:**
   - Test 1 (Customer Project Scaffolding & Idempotency):
     - First run in customer project directory created compliant README (SHA-256: `4485a0eca721...`).
     - Second in-place run (`bash scripts/install_pipeline.sh .`) resulted in identical file hash (`4485a0eca721...`). No clobbering or rewriting occurred.
     - Custom section additions (`## 5. Customer Specific Operational Constraints`) were preserved across in-place runs.
   - Test 2 (Legacy Circular Customer README Migration):
     - Simulated legacy README with `git clone https://gitlab.com/uas-safety/uav-011.git ./.tmp-pipeline`.
     - `install_pipeline.sh` detected `git clone.*\.tmp-pipeline`, triggered scaffolding, and upgraded the file to the clean customer format.
     - Subsequent in-place run on the upgraded README confirmed complete idempotency.
   - Adversarial Tests:
     - Real-world GitLab remote URL handling: clean upgrade, zero non-existent domain URLs, full idempotency.
     - Read-only (`chmod 444`) README handling: safely bypassed re-scaffolding without corruption or error.
     - 3 legacy circular variants (missing role tag, missing maintenance section, circular command in custom text): all detected and upgraded.

3. **Core Test Suite & Baseline Conformance:**
   - Command: `python3 -m unittest discover tests`
     - Output: `Ran 23 tests in 6.756s ... OK` (Exit code: 0).
   - Command: `python3 scripts/verify_downstream_baseline.py --no-domain`
     - Output: All 30 checks verified cleanly.
     - Output: `Success: Build and test suite execution passed for '/Users/perkunas/jail/DEAP01-spec-core'. Conformance gate verified.` (Exit code: 0).

---

## 2. Logic Chain

1. **From Observation 1:** `scripts/install_pipeline.sh` establishes an explicit branch in `SHOULD_SCAFFOLD_README` evaluation based on `TARGET_ROLE`.
2. For `DOWNSTREAM_CUSTOMER_PROJECT`, it checks whether `git clone.*\.tmp-pipeline` is present, or whether required sections (`DOWNSTREAM_CUSTOMER_PROJECT`, `Project Lifecycle & Tooling Maintenance`) are absent.
3. If the README is already compliant, `SHOULD_SCAFFOLD_README` evaluates to `false`. Therefore, the existing file is left untouched on subsequent runs, guaranteeing idempotence (supported by Observation 2, Test 1).
4. If a customer README is from a legacy circular installation (carrying `git clone.*\.tmp-pipeline`), `SHOULD_SCAFFOLD_README` evaluates to `true`, replacing the circular instructions with project-specific tooling commands (`bash scripts/install_pipeline.sh .`) (supported by Observation 2, Test 2).
5. All baseline conformance and regression tests continue to pass with zero defects (supported by Observation 3).

---

## 3. Caveats

- Live network connectivity to external git hosting services was not exercised during local test harness runs; git operations operated in offline/local mock mode.

---

## 4. Conclusion

The execution stability and idempotence of `scripts/install_pipeline.sh` are empirically verified.
Customer project scaffolding, in-place update idempotence, legacy circular README migration, and core baseline test suites all perform correctly.
**Verdict: APPROVE.**

---

## 5. Verification Method

To independently verify these conclusions:

1. Run core unit tests:
   ```bash
   python3 -m unittest discover tests
   ```
2. Run downstream baseline verification:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
3. Test customer scaffolding and idempotence:
   ```bash
   TMP_DIR=$(mktemp -d)
   bash scripts/install_pipeline.sh "$TMP_DIR/uav-test"
   HASH1=$(shasum -a 256 "$TMP_DIR/uav-test/README.md" | awk '{print $1}')
   (cd "$TMP_DIR/uav-test" && bash scripts/install_pipeline.sh .)
   HASH2=$(shasum -a 256 "$TMP_DIR/uav-test/README.md" | awk '{print $1}')
   test "$HASH1" = "$HASH2" && echo "IDEMPOTENT"
   rm -rf "$TMP_DIR"
   ```
