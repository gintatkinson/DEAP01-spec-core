# Milestone 2 Review & Adversarial Audit Report: `scripts/install_pipeline.sh`

**Reviewer Agent:** `reviewer_m2_1`  
**Roles:** Reviewer, Adversarial Critic  
**Target:** `scripts/install_pipeline.sh` (Milestone 2 implementation by `worker_m2_1`)  
**Date:** 2026-09-21  

---

## 1. Quality Review Summary

**Verdict**: **APPROVE**

The implementation in `scripts/install_pipeline.sh` fulfills all requirements of Milestone 2 (R2 and R3 from user request `2026-09-21T16:32:10Z`):
1. **Dynamic Repository Role Detection**: Implemented `-r, --role ROLE` option parsing with robust normalization (`tr '[:upper:]' '[:lower:]' | tr '-' '_'`), validation against permitted roles, and clear error exit (exit code 1) on invalid or missing role arguments. Auto-detection deterministically identifies `DOMAIN_DISTRIBUTION_TEMPLATE` (via `DEAP-*` directory names, git remote URLs, domain options) versus `DOWNSTREAM_CUSTOMER_PROJECT` (via `uav-*` directory names or default fallback).
2. **Distinct README Scaffolding**:
   - `DOMAIN_DISTRIBUTION_TEMPLATE`: Explicitly declares role `DOMAIN_DISTRIBUTION_TEMPLATE`, documents the Clean Landing Zone Invariant in Section 1.1, provides the turnkey customer project onboarding clone command (`git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ...`), and documents in-place domain template tooling updates (`bash scripts/install_pipeline.sh .`).
   - `DOWNSTREAM_CUSTOMER_PROJECT`: Explicitly declares role `DOWNSTREAM_CUSTOMER_PROJECT`, documents Customer Application Workspace Scope in Section 1.1, completely eliminates circular self-cloning commands, documents downstream baseline verification (`python3 scripts/verify_downstream_baseline.py --no-domain`), Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py` and `compile_sysml.py --compile`), and in-place tooling updates (`bash scripts/install_pipeline.sh .`).
   - Both scaffolding variants strictly preserve Section 4 (Operator Prompt Catalog) and Section 5 (Verification & Quality Gates).
3. **Role-Aware README Regeneration**: Replaces previous fragile checks with a role-aware `SHOULD_SCAFFOLD_README` evaluation. Compliant READMEs for both domain templates and customer projects are preserved without redundant overwrites, while uninitialized, GitLab boilerplate, or legacy circular customer READMEs are automatically upgraded.
4. **Shell Syntax Hygiene**: All generated code fences contain valid shell syntax (`bash -n` verified) with zero unescaped parentheses in comments and zero unquoted angle brackets.

---

## 2. Findings

### Good Practices Observed
- **Case and Delimiter Normalization**: CLI role argument normalization uses `tr '[:upper:]' '[:lower:]' | tr '-' '_'` to gracefully handle various user input formats (`domain-template`, `DOMAIN_DISTRIBUTION_TEMPLATE`, `customer_project`, `customer-project`).
- **Comprehensive Flag Parsing**: Handles both `-r <val>`, `--role <val>`, `--role=<val>`, and `-r=<val>` variants uniformly.
- **Strict Boundary Locking**: In-place installer operations strictly refuse execution if targeting the upstream compiler itself (`INSTALLER_ROOT/.pipeline/upstream` check), while allowing in-place execution within initialized downstream templates and customer workspaces.
- **Idempotence & Upgradeability**: The regeneration condition handles both idempotence (preserving user modifications in compliant READMEs) and migration (automatically upgrading legacy READMEs with circular instructions).

### No Critical or Major Defects Found
- No integrity violations detected (no hardcoded test results, no facade implementations, no shortcuts, no fabricated outputs).

---

## 3. Verified Claims

| Claim from `worker_m2_1` | Verification Method | Result |
| :--- | :--- | :--- |
| `bash -n scripts/install_pipeline.sh` exits 0 | Executed `bash -n scripts/install_pipeline.sh` in workspace | **PASS** (Exit 0) |
| `python3 -m unittest discover tests` passes (23 tests) | Executed in workspace | **PASS** (23 tests OK) |
| `python3 scripts/verify_downstream_baseline.py --no-domain` passes | Executed in workspace (30 checks) | **PASS** (Exit 0, all 30 checks OK) |
| `-r, --role` validates against invalid roles | Executed `--role invalid_role` in temp dir | **PASS** (Exit 1 with descriptive stderr) |
| `--role` requires argument | Executed `--role` without argument | **PASS** (Exit 1 with descriptive stderr) |
| Auto-detects `DEAP-*` as `DOMAIN_DISTRIBUTION_TEMPLATE` | Executed installer on `DEAP-uas-safety` dir | **PASS** (Role declared, landing zone invariant documented) |
| Auto-detects `uav-*` as `DOWNSTREAM_CUSTOMER_PROJECT` | Executed installer on `uav-011` dir | **PASS** (Role declared, zero circular clone commands) |
| Auto-detects arbitrary directory as `DOWNSTREAM_CUSTOMER_PROJECT` | Executed installer on `my-flight-control` dir | **PASS** (Customer project scaffolded) |
| Compliant customer README is not overwritten (idempotence) | Appended marker to compliant README, re-ran installer | **PASS** (Marker preserved, file untouched) |
| Legacy circular customer README is upgraded | Created legacy circular README, re-ran installer | **PASS** (Circular clone removed, upgraded) |
| Shell code blocks in scaffolded READMEs pass `bash -n` | Extracted all ```bash blocks from generated READMEs and checked syntax | **PASS** (`bash -n` exit 0 on all blocks) |
| Zero unescaped parens in bash comments | Scanned all comments in generated README blocks | **PASS** (Zero unescaped parens) |

---

## 4. Adversarial Challenge & Stress-Testing

**Overall Risk Assessment:** **LOW**

### Stress Test Matrix

| Scenario / Attack Vector | Predicted Risk | Actual Behavior | Result |
| :--- | :--- | :--- | :--- |
| **CLI Argument Injection**: Missing or invalid role flag (`--role` or `-r` without arg, or `--role invalid`) | Unhandled exception or silent fallback to wrong role | Script cleanly aborts with `exit 1` and descriptive stderr message | **PASS** |
| **Heredoc Backtick Escaping**: Heredoc `\`\`\`bash` expansion inside unquoted `cat << EOF` | Stray escape slashes or corrupt code fences in output markdown | Escaped backticks resolve to standard markdown fences (```` ```bash ````), valid for rendering | **PASS** |
| **Customer Self-Clone Elimination**: Regex search for `git clone.*\.tmp-pipeline` in customer project README | Customer workspace README still contains circular instructions | Zero matches in customer project README; instructions replaced with verification & Level 0 ingestion | **PASS** |
| **Landing Zone Invariant Documentation**: Check whether `DOMAIN_DISTRIBUTION_TEMPLATE` README retains invariant | Domain template lacks landing zone documentation | Section 1.1 explicitly details clean landing zone invariant across `schema/` and `docs/` | **PASS** |
| **Subagent Prompt Catalog Integrity**: Check whether appending Section 4 and 5 damages prompt catalog | Truncation or divergence in subagent instructions | Operator Prompt Catalog and Quality Gates preserved verbatim across both template variants | **PASS** |
| **In-Place Update Idempotence**: Repetitive executions of `bash scripts/install_pipeline.sh .` | Customer or domain template customizations wiped on routine updates | `SHOULD_SCAFFOLD_README` evaluates to `false` on compliant READMEs, preserving existing content | **PASS** |

---

## 5. Coverage Gaps & Unverified Items

- **Coverage Gaps**: None. All functional requirements (R2, R3) and verification commands from Milestone 2 were directly exercised.
- **Unverified Items**: None.

---

## 6. Final Verdict

**APPROVE**. The Milestone 2 changes in `scripts/install_pipeline.sh` are robust, syntactically and logically clean, fully compliant with DEAP architectural boundaries, and ready for integration.
