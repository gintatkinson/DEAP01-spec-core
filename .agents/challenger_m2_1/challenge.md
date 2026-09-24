# Adversarial Challenge Report: `scripts/install_pipeline.sh`

## Challenge Summary

**Overall risk assessment**: LOW  
**Verdict**: **APPROVE**

This adversarial challenge evaluated `scripts/install_pipeline.sh` against the requirements in `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` (§ R2, § R3). A hermetic, automated stress test harness (`/tmp/test_challenger_install.py`) executed 18 adversarial scenarios across isolated temporary directories (`tempfile.TemporaryDirectory()`) completely outside the repository.

All 18 stress test cases passed empirically. The implementation robustly handles role auto-detection, explicit CLI `--role` arguments (including syntax variants and overrides), error boundaries, existing README remediation, and generates syntactically valid Markdown/Shell blocks with zero circular clones, zero unescaped parentheses in comments, and zero unquoted angle-bracket placeholders.

---

## Challenges

### [Low] Challenge 1: Fallback Detection when Directory Name is Non-Standard and `--role` is Omitted

- **Assumption challenged**: Downstream users may initialize a project in an arbitrarily named folder (e.g. `my-flight-project`) without passing `--role`.
- **Attack scenario**: The installer script must safely default to a valid role without raising unbound variable errors or generating contradictory scaffolding.
- **Blast radius**: If undefined, README generation would fail or crash with shell errors.
- **Observed Behavior**: Lines 333-342 of `scripts/install_pipeline.sh` inspect `.pipeline/lineage.json` and fall back cleanly to `DOWNSTREAM_CUSTOMER_PROJECT`.
- **Stress Test Evidence**: Verified in Test 4 and fallback checks; target repository role is deterministically set to `DOWNSTREAM_CUSTOMER_PROJECT`.
- **Mitigation**: Already implemented in `scripts/install_pipeline.sh:341`.

### [Low] Challenge 2: Legacy Customer Project README Remediation

- **Assumption challenged**: Customer project repositories that were previously scaffolded with the buggy circular onboarding command (`git clone ... uav-011`) might retain the circular command upon running `bash scripts/install_pipeline.sh .` because `README.md` already exists on disk.
- **Attack scenario**: If the installer skips scaffolding whenever `README.md` exists, customer repositories updating their tooling would remain permanently poisoned with circular clone instructions.
- **Blast radius**: Customer onboarding documentation remains broken and circular.
- **Observed Behavior**: Lines 591-597 specifically inspect whether `git clone.*\.tmp-pipeline` is present in an existing `DOWNSTREAM_CUSTOMER_PROJECT` README, and sets `SHOULD_SCAFFOLD_README=true` to automatically regenerate and purge the circular command.
- **Stress Test Evidence**: Tested empirically in Test 9; a legacy README with circular clone was completely overwritten with the proper customer project README containing `bash scripts/install_pipeline.sh .` and zero `.tmp-pipeline` references.
- **Mitigation**: Robust regex triggers in `scripts/install_pipeline.sh:592`.

---

## Stress Test Results

| Test ID | Scenario | Expected Behavior | Actual Behavior | Result |
|---|---|---|---|---|
| **Test 1** | Target dir named `DEAP-uas-infrastructure-safety` (auto-detect domain template) | Detects `DOMAIN_DISTRIBUTION_TEMPLATE`, generates onboarding clone command and Clean Landing Zone Invariant | Detected role `DOMAIN_DISTRIBUTION_TEMPLATE`, README contains onboarding clone and invariant | **PASS** |
| **Test 2** | Target dir named `uav-011` (auto-detect customer project) | Detects `DOWNSTREAM_CUSTOMER_PROJECT`, generates zero circular clone commands, zero `.tmp-pipeline` commands, contains in-place update | Detected role `DOWNSTREAM_CUSTOMER_PROJECT`, ZERO circular clones, contains `bash scripts/install_pipeline.sh .` | **PASS** |
| **Test 3** | Explicit `--role domain-template` on custom folder | Forces `DOMAIN_DISTRIBUTION_TEMPLATE`, generates clean landing zone and onboarding command | Scaffolds `DOMAIN_DISTRIBUTION_TEMPLATE` README with onboarding command | **PASS** |
| **Test 4** | Explicit `--role customer-project` on custom folder | Forces `DOWNSTREAM_CUSTOMER_PROJECT`, generates zero circular clones | Scaffolds `DOWNSTREAM_CUSTOMER_PROJECT` README with zero circular clones | **PASS** |
| **Test 5** | Explicit `--role invalid-role` | Exits cleanly with code 1 and descriptive error on stderr | Return code 1, stderr: `Error: Invalid --role 'invalid-role'. Valid values: ...` | **PASS** |
| **Test 6.1** | Syntax variant `--role=domain-template` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | **PASS** |
| **Test 6.2** | Syntax variant `--role=customer-project` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | **PASS** |
| **Test 6.3** | Syntax variant `-r domain-template` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | **PASS** |
| **Test 6.4** | Syntax variant `-r customer-project` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | **PASS** |
| **Test 6.5** | Syntax variant `-r=domain-template` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | Exit 0, role = `DOMAIN_DISTRIBUTION_TEMPLATE` | **PASS** |
| **Test 6.6** | Syntax variant `-r=customer-project` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | Exit 0, role = `DOWNSTREAM_CUSTOMER_PROJECT` | **PASS** |
| **Test 7.1** | Missing argument `--role` | Exit 1, stderr: `Error: --role requires a role argument` | Exit 1, stderr: `Error: --role requires a role argument ('domain-template' or 'customer-project').` | **PASS** |
| **Test 7.2** | Missing argument `-r` | Exit 1, stderr: `Error: -r requires a role argument` | Exit 1, stderr: `Error: -r requires a role argument ('domain-template' or 'customer-project').` | **PASS** |
| **Test 7.3** | Flag boundary `--role --provider gitlab` | Exit 1, error rejecting next flag as argument | Exit 1, stderr: `Error: --role requires a role argument` | **PASS** |
| **Test 7.4** | Flag boundary `-r -p gitlab` | Exit 1, error rejecting next flag as argument | Exit 1, stderr: `Error: -r requires a role argument` | **PASS** |
| **Test 8.1** | Explicit `--role domain-template` overriding `uav-011` directory name | Explicit flag takes precedence over directory name heuristic | Role resolved to `DOMAIN_DISTRIBUTION_TEMPLATE` | **PASS** |
| **Test 8.2** | Explicit `--role customer-project` overriding `DEAP-uas-safety` directory name | Explicit flag takes precedence over directory name heuristic | Role resolved to `DOWNSTREAM_CUSTOMER_PROJECT` | **PASS** |
| **Test 9** | Remediation of existing README with legacy circular clone | Replaces legacy circular clone with customer in-place update instructions | Existing `.tmp-pipeline` circular clone replaced with `DOWNSTREAM_CUSTOMER_PROJECT` tooling update | **PASS** |

---

## Code Fence & Syntax Integrity Analysis

All code fences across generated READMEs and the upstream compiler README were extracted and subjected to rigorous syntactic and lexical analysis:

1. **`bash -n` Syntax Verification**:
   - `DOMAIN_DISTRIBUTION_TEMPLATE` README: 8 shell code blocks tested -> **100% valid bash syntax (0 errors)**.
   - `DOWNSTREAM_CUSTOMER_PROJECT` README: 8 shell code blocks tested -> **100% valid bash syntax (0 errors)**.
   - `DEAP01-spec-core` README: 14 shell code blocks tested -> **100% valid bash syntax (0 errors)**.

2. **Unescaped Parentheses in Comments**:
   - Every comment line (starting with `#`) inside all shell code blocks was analyzed with `(?<!\\)[()]`.
   - Result: **Zero unescaped parentheses found** across all shell code blocks.

3. **Unquoted Angle-Bracket Placeholders**:
   - Every non-comment line inside all shell code blocks was checked for `<placeholder>` tokens outside quotes.
   - Result: **Zero unquoted angle-bracket placeholders found**. All clone URLs and file paths are fully resolved concrete strings.

4. **Project Conformance Gate**:
   - `python3 scripts/verify_downstream_baseline.py --no-domain` -> **PASSED all checks cleanly**.
   - `python3 -m unittest discover -s tests -p "test_*.py"` -> **23 tests run, 0 failures**.

---

## Unchallenged Areas

- **Live Remote Network Cloning**: Executing live `git clone` from external GitHub or GitLab servers was not run directly over the internet to preserve hermetic test isolation in sandbox environments; URL synthesis and regex formats were verified deterministically.

---

## Final Verdict

**APPROVE**. The modifications to `scripts/install_pipeline.sh` fully satisfy Requirements R2 and R3 with exceptional defensive engineering, strict error handling, and robust documentation generation.
