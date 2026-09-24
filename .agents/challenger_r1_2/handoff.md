# Empirical Challenger 2 Handoff Report: Downstream README Scaffolding & Onboarding Verification

**Agent:** Challenger 2 (`.agents/challenger_r1_2`)  
**Role:** Empirical Challenger / Adversarial Verifier  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Primary Commercial Toolchain Integration Context:** `MATLAB / Simulink / Stateflow / Embedded Coder`  
**Verdict:** `APPROVE`

---

## 1. Observation

### 1.1 Scope & Verification Target
Direct empirical verification and adversarial stress-testing were performed on `scripts/install_pipeline.sh` (specifically the downstream README scaffolding, URL resolution hierarchy, and onboarding shell blocks) and `README.md` across both Tier 1 (Upstream Compiler -> Domain Template) and Tier 2 (Domain Template -> Customer Workspace) boundaries.

### 1.2 Automated 17-Point Empirical Test Suite
An isolated end-to-end empirical testing harness was authored and executed in a dedicated sandbox (`/tmp/test_empiric_challenger2.py`), completely outside the workspace repository.

**Raw Test Suite Execution Output (Task-70):**
```
=== Starting Empirical Challenger 2 Test Suite ===
Running E2E Execution Test in Customer Git Repo...
Executing in customer git repo: git clone "/var/folders/1g/l0zx9f054xn2vkc4944jzzpr0000gp/T/tmp3jz70v8h/DEAP-uas-safety" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline

=== TEST RESULTS SUMMARY ===
[PASS] scaffolding_github_https: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_github_ssh: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_gitlab_https: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_gitlab_ssh: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_custom_gitlab_url_port: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_no_remote: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_no_remote_custom_gh_org: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_no_remote_custom_gitlab_group: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_remote_is_spec_core: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] scaffolding_no_git_directory: Scaffolded correctly, valid syntax, zero angle brackets, zero unescaped parens
[PASS] e2e_onboarding_command_extraction: Successfully extracted onboarding command
[PASS] e2e_onboarding_command_structure: Command format validated: git clone https://github.com/my-defense-org/DEAP-uas-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
[PASS] e2e_onboarding_execution_git_repo: Onboarding executed cleanly; files copied, .tmp-pipeline cleaned up, baseline passed
[PASS] e2e_onboarding_execution_non_git: Onboarding executed cleanly in non-git directory with exit code 0
[PASS] e2e_preexisting_schema_execution: Pre-existing schema preserved, domain models copied, no nested schema/schema
[PASS] e2e_in_place_update: In-place update executed cleanly with exit code 0
[PASS] spec_core_readme_syntax: All 17 shell blocks in spec-core README pass syntax, zero angle brackets, zero unescaped parens in comments
[PASS] spec_core_baseline_verification: scripts/verify_downstream_baseline.py --no-domain passes cleanly in DEAP01-spec-core

OVERALL VERDICT: APPROVE
```

### 1.3 Regex & Syntax Checks
- **Angle Brackets (`<...>`):**
  A full AST and regex scan (`r'<[^>\n]+>'`) was performed across all shell code blocks in every generated `README.md` and `DEAP01-spec-core/README.md`. Exactly 0 instances of unquoted angle bracket placeholders were found.
- **Unescaped Parentheses in Shell Comments:**
  Regex scan (`r'(?<!\\)[\(\)]'`) over all `#` comment lines within fenced shell blocks detected exactly 0 unescaped parentheses.
- **Shell Syntax (`bash -n`):**
  Every shell block across all test variants passed `bash -n` syntax validation with exit code 0.
- **Zsh & Whitespace Path Compatibility:**
  Executing the extracted onboarding command under `zsh` with paths containing whitespace (e.g. `/tmp/test dir with spaces/customer project`) executed with exit code 0 and completed successfully.

---

## 2. Logic Chain

1. **Remote Scaffolding Hierarchy (R2 Verification):**
   - Observations 1.2 demonstrate that `scripts/install_pipeline.sh` correctly resolves the domain remote across GitHub HTTPS, GitHub SSH, GitLab HTTPS, GitLab SSH, and self-hosted GitLab with custom port numbers.
   - When no remote is configured, it falls back to concrete URLs without angle brackets (`https://github.com/your-org/<name>.git` or user-supplied `--github-org` / `--gitlab-group`).
   - When the target directory's remote is `DEAP01-spec-core`, it refuses to leak `DEAP01-spec-core.git` into the domain README, falling back safely to the project name.
   - In Tier 2 installations, the installer correctly inspects `$INSTALLER_ROOT` (which is the cloned domain template without `.pipeline/upstream`), extracting the domain template's remote URL and preserving it in the customer workspace README.

2. **Onboarding Command Validity (R2 Verification):**
   - The generated onboarding command:
     `git clone <domain-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
     was extracted verbatim from generated READMEs.
   - When executed in a clean customer repository, the command successfully cloned the repository, executed the installer, installed all baseline governance and skills, removed `.pipeline/upstream`, and deleted the temporary directory `./.tmp-pipeline`.
   - The command operates completely self-contained with zero external sibling path references (`../...`).

3. **Schema Preservation & Idempotence (R3 Verification):**
   - Tested onboarding into a customer workspace with a pre-existing `schema/` directory containing both `.gitkeep` and pre-existing customer models (`PREEXISTING_CUSTOMER.sysml`).
   - Verified that `schema/` contents from the domain template (`UAS_TACTICAL_SAFETY.sysml`) were copied alongside existing files, preserving existing customer models, and creating no nested `schema/schema/` directory.
   - In-place updates (`bash scripts/install_pipeline.sh .`) executed with exit code 0 without errors or corruption.

---

## 3. Caveats

1. **Baseline Verification in Non-Git Environments:**
   - The onboarding command itself (`git clone ... && bash ... && rm -rf ...`) runs with exit code 0 in any directory, git or non-git.
   - However, subsequent execution of `python3 scripts/verify_downstream_baseline.py --no-domain` requires the directory to be a git repository (`git rev-parse --is-inside-work-tree`) to place the `restoration-point` tag. When run in a git repository (or after running `git init`), the baseline verification passes cleanly with exit code 0.

---

## 4. Conclusion

All empirical stress tests passed unconditionally:
- Downstream README scaffolding robustly handles all remote variants.
- The onboarding command is 100% syntactically valid shell code in both `bash` and `zsh`, cleanly handling clean repos, pre-existing schemas, and spaced paths.
- Zero angle brackets (`<...>`) exist in generated shell blocks.
- Zero unescaped parentheses exist in shell comments.
- Upstream baseline verification passes cleanly.

**Final Verdict: APPROVE**

---

## 5. Verification Method

To independently verify these results, run the self-contained test script in `/tmp`:

```bash
python3 /tmp/test_empiric_challenger2.py
```
Or execute the spec-core baseline verification:
```bash
python3 scripts/verify_downstream_baseline.py --no-domain
```
