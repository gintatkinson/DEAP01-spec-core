# Independent Quality & Adversarial Review Report: Milestone 2 (R2 and R3)

**Agent:** `reviewer_m2_2`  
**Role:** Reviewer & Adversarial Critic  
**Scope:** Review `scripts/install_pipeline.sh` (M2: Distinct README scaffolding, dynamic role detection, shell script robustness, template escaping, backward compatibility, test verification)  
**Date:** 2026-09-21T16:58:30Z  

---

## Review Summary

**Verdict**: **APPROVE**  
**Overall Risk Assessment**: **LOW**

The implementation in `scripts/install_pipeline.sh` provided by `worker_m2_1` satisfies all functional and architectural requirements specified under Milestone 2 (R2 and R3) in `.agents/ORIGINAL_REQUEST.md`:
1. Clean decoupling and dynamic detection of repository roles (`DOMAIN_DISTRIBUTION_TEMPLATE` vs `DOWNSTREAM_CUSTOMER_PROJECT`) via explicit `-r, --role` CLI parameters and intelligent fallback heuristics.
2. Distinct README scaffolding for domain distribution templates (declaring clean landing zone invariant and single-line customer clone command) vs customer application workspaces (declaring customer delivery scope, downstream baseline verification, Level 0 OEM Ground Truth ingestion, in-place tooling update, and zero circular self-cloning commands).
3. Robust shell script hygiene: all generated code fences contain pure valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.
4. Preserved backward compatibility across existing CLI parameters (`--provider`, `--platform`, `--tracker`, `--gitlab-url`, `--gitlab-group`, `--github-org`, `--jira-url`, `--jira-project`, `--jira-email`, `--domain-url`, `--domain-name`, positional `[TARGET_DIR]`).
5. Fully passing baseline and unit test suites with zero regressions.

No integrity violations, hardcoded test results, facade implementations, or bypasses were detected.

---

## Findings

### [Minor] Finding 1: CLI `--dest` Parameter Mention in Dispatch vs Positional `[TARGET_DIR]`
- **What**: The review dispatch mentioned checking backward compatibility for `(--domain-url, --provider, --dest, etc.)`. However, `scripts/install_pipeline.sh` does not support `--dest` or `-d`; destination has always been specified via the positional argument `[TARGET_DIR]`.
- **Where**: `scripts/install_pipeline.sh:19` (`Usage: install_pipeline.sh [OPTIONS] [TARGET_DIR]`) and lines 188-204.
- **Why**: Passing an unknown flag like `--dest /path` triggers the unknown option error handler (`Error: Unknown option: --dest`) and exits with code 1.
- **Suggestion**: Since git history confirms `--dest` was never an existing CLI argument (all repository scripts and documentation use positional `TARGET_DIR`), this is not a regression. If desired in a future enhancement, `--dest` / `-d` could be added as an explicit alias for `TARGET_DIR`.

### [Minor] Finding 2: Auto-Detection Priority for `--domain-url`
- **What**: When `--role` is omitted, the presence of `--domain-url` in line 329 causes `TARGET_ROLE` to resolve to `DOMAIN_DISTRIBUTION_TEMPLATE`, even if the target directory basename is not prefixed with `DEAP-`.
- **Where**: `scripts/install_pipeline.sh:325-332`.
- **Why**: Specifying `--domain-url` is explicitly intended for scaffolding a domain distribution template with an authoritative clone URL. However, if a user invokes `install_pipeline.sh --domain-url <URL> /path/to/uav-011` without specifying `--role`, the script infers domain template mode.
- **Suggestion**: The user can override this by explicitly specifying `--role customer-project`. For clarity, this behavior is well-documented in the help text and regression tests.

---

## Verified Claims

1. **Bash Syntax Integrity**:
   - `bash -n scripts/install_pipeline.sh` → PASSED (exit code 0).
2. **Unit Test Suite**:
   - `python3 -m unittest discover tests` → PASSED (23 tests in 6.6s, OK).
3. **Downstream Baseline Conformance**:
   - `python3 scripts/verify_downstream_baseline.py --no-domain` → PASSED (All 30 checks verified, exit code 0).
4. **Distinct README Scaffolding by Role**:
   - `DOMAIN_DISTRIBUTION_TEMPLATE`: Header `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\``, Clean Landing Zone Invariant section, customer onboarding clone command `git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`. Verified via hermetic test → PASSED.
   - `DOWNSTREAM_CUSTOMER_PROJECT`: Header `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``, Customer Application Workspace Scope section, verification command `python3 scripts/verify_downstream_baseline.py --no-domain`, Level 0 ingestion command `sysmlv2_ingest.py`, and in-place update `bash scripts/install_pipeline.sh .`. Zero circular clone commands found. Verified via hermetic test → PASSED.
5. **Code Block Quoting & Shell Comment Escaping**:
   - Every ` ```bash ` block generated in both templates was extracted and checked with `bash -n`.
   - All comments verified to contain zero unescaped parentheses.
   - Zero unquoted angle-bracket placeholders found. Verified via independent script → PASSED.
6. **Idempotence & Upgradeability**:
   - Running `install_pipeline.sh` multiple times on a compliant project preserves custom additions without overwriting.
   - Running `install_pipeline.sh` on a legacy customer README with circular `.tmp-pipeline` commands successfully triggers re-scaffolding and upgrades to the clean customer template. Verified via hermetic test → PASSED.
7. **CLI Parameter Robustness & Error Handling**:
   - Missing required argument for `--role`, `--provider`, `--domain-url`, etc. emits descriptive stderr and exits with code 1.
   - Invalid role value emits descriptive stderr and exits with code 1.
   - Unexpected positional arguments emit error and exit with code 1.
   - Spaced path handling (`/tmp/test space project`) functions without path split errors. Verified via hermetic test → PASSED.

---

## Adversarial Stress-Testing & Challenges

### Challenge 1: Target Repository is the Pipeline Root Repository Itself
- **Assumption**: The script should refuse to operate in-place if executed on `DEAP01-spec-core` when `.pipeline/upstream` exists.
- **Attack Scenario**: Run `./scripts/install_pipeline.sh .` from repository root.
- **Result**: Refused with exit code 1: `REFUSING: target is the pipeline repository itself, not a downstream project.` → PASSED.

### Challenge 2: Role Override Precedence
- **Assumption**: An explicit `--role` CLI flag should override directory name heuristics.
- **Attack Scenario**: Run with `--role domain-template` on directory named `uav-011`, and `--role customer-project` on directory named `DEAP-uas-infrastructure-safety`.
- **Result**: Explicit flag takes absolute precedence; `uav-011` receives `DOMAIN_DISTRIBUTION_TEMPLATE` and `DEAP-uas-infrastructure-safety` receives `DOWNSTREAM_CUSTOMER_PROJECT` when explicitly requested. → PASSED.

### Challenge 3: Escaping in Unquoted Here-Docs
- **Assumption**: Code blocks inside `cat << EOF` in lines 747-898 must escape all backticks (`\`\`\``) so they are not evaluated as command substitution during generation.
- **Attack Scenario**: Check whether any backticks or variables in generated bash code blocks expand erroneously during generation.
- **Result**: All backticks in lines 787-880 are properly escaped (`\`\`\`bash` and `\`\`\``). Section 4 uses a quoted here-doc (`cat << 'EOF' >> ...`) which safely suppresses expansion of all internal tokens. → PASSED.

---

## Coverage Gaps

- None. All requirements R2 and R3 and associated edge cases were verified across both unit tests and hermetic execution runs.

---

## Unverified Items

- None. All targeted functionality and constraints have been verified independently.
