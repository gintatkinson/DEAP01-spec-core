# Empirical Challenge Report: Installer Execution Stability, Idempotence & Scaffolding

**Target Subsystem:** `scripts/install_pipeline.sh`  
**Challenger Role:** Empirical Challenger (`challenger_m2_2`)  
**Milestone:** M2.2 Verification (Issue #363 / ORIGINAL_REQUEST.md § R2 & § R3)  
**Verdict:** **APPROVE**  

---

## 1. Challenge Summary

**Overall Risk Assessment:** **LOW**

The implementation in `scripts/install_pipeline.sh` successfully addresses the two-tier architectural distinction between Tier 1 Domain Distribution Templates (`DOMAIN_DISTRIBUTION_TEMPLATE`) and Tier 2 Customer Application Workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`).

All adversarial stress tests, idempotency checks, and legacy circular README migration checks passed with 100% success across all scenarios:
1. **Initial Customer Scaffolding:** First run in a customer project directory properly scaffolds the clean customer README with role `DOWNSTREAM_CUSTOMER_PROJECT`, Section 3 `Project Lifecycle & Tooling Maintenance`, in-place update command (`bash scripts/install_pipeline.sh .`), baseline verification command (`python3 scripts/verify_downstream_baseline.py --no-domain`), and zero circular clone commands (`git clone ... .tmp-pipeline`).
2. **In-Place Idempotence:** Subsequent execution (`bash scripts/install_pipeline.sh .`) in an initialized customer project is strictly idempotent. The customer `README.md` is neither clobbered, overwritten, nor altered (verified via SHA-256 byte-by-byte parity). Furthermore, custom customer sections appended to `README.md` are completely preserved.
3. **Legacy Circular README Detection & Upgrade:** A customer repository containing a legacy circular onboarding README (with `git clone.*\.tmp-pipeline` or lacking `DOWNSTREAM_CUSTOMER_PROJECT` / `Project Lifecycle & Tooling Maintenance`) is detected deterministically and safely upgraded to the clean customer README. Subsequent runs on the upgraded repository are fully idempotent.
4. **Test Suite & Baseline Gates:** All 23 unittests in `python3 -m unittest discover tests` and all 30 checks in `python3 scripts/verify_downstream_baseline.py --no-domain` pass with zero failures.

---

## 2. Adversarial Challenges & Test Matrix

### Challenge 1: Idempotency Under In-Place Customer Update
- **Assumption Challenged:** Re-running `bash scripts/install_pipeline.sh .` inside an installed customer application workspace must not clobber or alter an existing compliant customer `README.md`, nor strip user-added custom operational sections.
- **Attack Scenario:**
  - Initialized customer project `/tmp/uav_cust_test_*/uav-011`.
  - Executed initial pipeline installation via `scripts/install_pipeline.sh`.
  - Recorded SHA-256 hash of generated `README.md`.
  - Executed in-place update `bash scripts/install_pipeline.sh .`.
  - Added custom section (`## 5. Customer Specific Operational Constraints`) and executed in-place update again.
- **Result:** **PASS**. `README.md` SHA-256 hash was completely invariant on rerun. Custom additions were preserved with zero alterations.

### Challenge 2: Detection and Safe Upgrade of Legacy Circular Customer READMEs
- **Assumption Challenged:** Existing customer repositories created prior to Issue #363 contain circular instructions telling the customer to clone themselves into `.tmp-pipeline`. Running `install_pipeline.sh` must identify this corrupted pattern and safely upgrade the README to the non-circular customer standard.
- **Attack Scenario:**
  - Synthesized customer project `/tmp/uav_legacy_test_*/uav-011` configured with a simulated legacy circular README containing `git clone https://gitlab.com/uas-safety/uav-011.git ./.tmp-pipeline`.
  - Executed `scripts/install_pipeline.sh`.
- **Result:** **PASS**. The installer detected the circular pattern (`grep -qE "git clone.*\.tmp-pipeline"`), triggered scaffolding, and replaced the circular section with Section 3 `Project Lifecycle & Tooling Maintenance` (`bash scripts/install_pipeline.sh .` and `python3 scripts/verify_downstream_baseline.py --no-domain`). All circular clone commands were eliminated. Subsequent in-place execution verified full idempotence.

### Challenge 3: Real-World Customer Repository with GitLab Remote
- **Assumption Challenged:** A customer repository configured with a remote origin pointing to GitLab (`https://gitlab.com/uas-safety/uav-011.git`) might cause the installer to mistakenly synthesize circular or invalid GitLab domain URLs.
- **Attack Scenario:**
  - Created git repository with `origin = https://gitlab.com/uas-safety/uav-011.git` and legacy circular README.
  - Executed `scripts/install_pipeline.sh`.
  - Re-executed in-place with `--provider gitlab`.
- **Result:** **PASS**. Customer README was upgraded to clean customer format without any circular URLs. In-place rerun was completely idempotent.

### Challenge 4: File Permission Stress (Read-Only README)
- **Assumption Challenged:** If a customer project's `README.md` has read-only permissions (`chmod 444`), in-place runs must not crash or corrupt the file.
- **Attack Scenario:** Set `README.md` to `0o444` and executed `bash scripts/install_pipeline.sh .`.
- **Result:** **PASS**. Installer executed `chmod u+w` defensively, verified compliance, bypassed re-scaffolding, and preserved file integrity without errors.

### Challenge 5: Legacy Circular README Variant Permutations
- **Assumption Challenged:** Variations in legacy customer READMEs (e.g. missing `DOWNSTREAM_CUSTOMER_PROJECT`, missing `Project Lifecycle & Tooling Maintenance`, or containing circular clone in a custom section) must all be detected and remediated.
- **Attack Scenario:** Tested 3 distinct structural variants across isolated temporary environments.
- **Result:** **PASS**. All 3 variants were correctly detected and upgraded to the clean customer README standard.

### Challenge 6: Baseline Verification and Core Unit Tests
- **Verification Commands:**
  - `python3 -m unittest discover tests` -> 23 tests, 0 failures, 0 errors.
  - `python3 scripts/verify_downstream_baseline.py --no-domain` -> All 30 conformance checks passed cleanly.
- **Result:** **PASS**.

---

## 3. Empirical Stress Test Execution Evidence

```
=== [TEST 1] Customer Project README Scaffolding & Idempotence ===
  -> Run 1: Initial pipeline installation into customer project...
     Run 1 produced valid customer README (SHA256: 4485a0eca721...)
  -> Run 2: In-place pipeline update (bash scripts/install_pipeline.sh .)...
     Run 2 idempotence verified: README.md was NOT clobbered or altered!
  -> Run 3: Adding customer custom section and running in-place update...
     Run 3 verified: Custom customer additions are preserved; compliant README is not clobbered!
  -> [TEST 1 PASSED]

=== [TEST 2] Legacy Circular Customer README Detection & Upgrade ===
  -> Step 1: Created simulated legacy circular customer README.
  -> Step 2: Running install_pipeline.sh on directory with legacy circular README...
     Legacy circular README successfully detected and safely upgraded!
  -> Step 3: Verifying subsequent run is idempotent on upgraded README...
     Subsequent run on upgraded README is fully idempotent!
  -> [TEST 2 PASSED]

=== [TEST 1-Variant] Generic Directory Name (non-uav prefix) Scaffolding & Idempotence ===
  -> [TEST 1-Variant PASSED]

=== [CONTRAST TEST] Domain Distribution Template Scaffolding & Idempotence ===
     Domain template correctly contains onboarding clone command and is idempotent!
  -> [CONTRAST TEST PASSED]

=== [ADV-1] Real-World Customer Project with GitLab Remote & Legacy Circular README ===
  -> Passed: Legacy circular README upgraded cleanly.
  -> Passed: In-place rerun is perfectly idempotent.
  -> Passed: Explicit provider does not clobber compliant README.

=== [ADV-2] Customer Project with Read-Only README (chmod 444) ===
  -> Passed: Read-only README handled gracefully without crash or alteration.

=== [ADV-3] Multiple Legacy Circular README Variants ===
  -> Passed: Circular clone in custom README detected and upgraded safely.
  -> Passed: Customer README missing DOWNSTREAM_CUSTOMER_PROJECT detected and upgraded safely.
  -> Passed: Customer README missing Project Lifecycle section detected and upgraded safely.
```

---

## 4. Unchallenged Areas

- Hardware-in-the-loop (HIL) physical drone testbeds (out of scope for spec compiler installation verification).
- Active live GitLab API network authentication (mocked/offline network test mode correctly handles tracker label provisioning gracefully).

---

## 5. Final Verdict

**APPROVE**. The implementation in `scripts/install_pipeline.sh` satisfies all requirements for execution stability, idempotency, legacy circular README migration, and test suite baseline verification.
