# Handoff Report: Verification & Testing Facilities Survey & Test Strategy

**From:** explorer_r4_3  
**To:** parent (fc7b047c-fd31-4577-ab8b-b65f4c56c828)  
**Date:** 2026-09-21T16:38:00Z  
**Type:** Hard (Task complete)  

---

## 1. Observation

1. **`scripts/verify_downstream_baseline.py` Checks and Structure**:
   - `run_all_checks(repo_root)` (lines 3474–3504) invokes Checks 10 through 30:
     - Check 10 (`check_gitignore_exists`): line 320.
     - Check 11 (`check_no_ds_store_files`): line 328.
     - Check 12 (`check_no_duplicate_master_blueprints`): line 377 (skips if `.pipeline/upstream` exists).
     - Check 13 (`check_latex_katex_syntax`): line 399.
     - Mermaid syntax (`check_mermaid_syntax`): line 3439.
     - Check 14 (`check_downstream_instructions_exist`): lines 505–545. Specifically:
       ```python
       with open(readme_path, "r", encoding="utf-8") as f:
           readme_content = f.read()
       if not (
           "# Downstream Cyber-Physical Infrastructure Safety Project" in readme_content
           or "Operator Prompt Catalog" in readme_content
       ):
           print(f"ERROR: Check 14 failed: README.md in '{repo_root}' lacks canonical downstream content...", file=sys.stderr)
           sys.exit(1)
       ```
     - Check 15 (`check_reconcile_backlog_tooling_exists`): line 546.
     - Check 16 (`check_upstream_template_clean_landing_zones`): line 560 (skips if `.pipeline/upstream` is absent).
     - Checks 17 through 30 (safety integrity, domain cleanliness, WBS suite, semantic AST parity, ICD completeness, etc.).
   - Gaps observed:
     - No shell syntax checker or `bash -n` validation for shell code blocks in any markdown file.
     - Zero checks for unescaped parentheses in comments within code fences (`# ... (...)`).
     - Zero checks verifying repository classification strings (`UPSTREAM_SPEC_CORE_COMPILER`, `DOMAIN_DISTRIBUTION_TEMPLATE`, `DOWNSTREAM_CUSTOMER_PROJECT`).

2. **`--no-domain` Behavior in `scripts/verify_downstream_baseline.py`**:
   - Lines 242–254:
     ```python
     no_domain_for_target = args.no_domain
     if not args.no_domain and (
         check_no_domain_config(repo_root) or check_no_domain_config(dest)
     ):
         no_domain_for_target = True
         flutter_domain = os.path.join(dest if is_flutter else repo_root, "lib", "domain")
         react_domain = os.path.join(dest if is_react else repo_root, "src", "domain")
         if os.path.isdir(flutter_domain) or os.path.isdir(react_domain):
             print(f"NOTE: Domain directory found on disk for '{dest}' -- overriding no_domain config and enabling domain verification.")
             no_domain_for_target = False
     ```
   - When `--no-domain` is passed on CLI, `no_domain_for_target = True` is preserved unconditionally.
   - For Flutter: skips checking `repository_resolver.dart` (line 3528), skips `_validate_domain_types` (line 3538), and skips build/test (`flutter pub get`, `flutter analyze`, `flutter test`, `flutter build macos --release`) (line 3544).
   - For React: skips checking validation file (line 3626), skips `_validate_domain_types` (line 3640), and skips build (`npm install`, `npm run build`) (line 3646).
   - Checks 10 through 30 (`run_all_checks`) still run in their entirety.

3. **Current Upstream `README.md`**:
   - Section 5.3 (lines 241–264) contains customer onboarding commands cloning domain repos:
     `git clone "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git" ./.tmp-pipeline && ...`
   - Section 5.4 (lines 273–361) contains an 80-line manual copy and inline Python monkeypatching script:
     `python3 -c "import os ... src = './.tmp-pipeline/AGENTS.md' ... transformed = content.replace(upstream_h, downstream_h) ... "` and `python3 -c "import json ... "`.

4. **Installer Scaffolding in `scripts/install_pipeline.sh`**:
   - Line 670 hardcodes:
     `> **Repository Role:** \`DOWNSTREAM_APPLICATION_WORKSPACE\``
   - Lines 703–706 generate customer onboarding commands targeting `$DOMAIN_REMOTE_URL`. When run in a customer project (e.g. `uav-011`), `$DOMAIN_REMOTE_URL` resolves to `uav-011`, generating circular clone commands.

5. **Existing Tests in `tests/`**:
   - `tests/test_domain_url_synthesis.py`: 251 lines, uses `unittest.TestCase` and `tempfile.TemporaryDirectory()`, invokes `scripts/install_pipeline.sh` via `subprocess.run(["bash", INSTALL_SCRIPT, ...])`, and asserts expected strings in generated `README.md`.
   - Running `python3 -m unittest discover tests` runs 23 tests in 6.45 seconds and passes with exit code 0.

---

## 2. Logic Chain

1. **Premise 1 (Testing Gap)**: `scripts/verify_downstream_baseline.py` validates repository file existence and domain types, but lacks checks for shell code fence execution validity, comment parentheses hygiene, and repository classification adherence.
2. **Premise 2 (Shell Parsing Friction)**: Unescaped parentheses in comments inside shell code blocks or command substitutions trigger syntax errors in zsh (`zsh: parse error in command substitution` / glob errors), and unquoted angle brackets `<placeholder>` trigger bash file redirection syntax errors.
3. **Premise 3 (Established Pattern)**: `tests/test_domain_url_synthesis.py` proves that `install_pipeline.sh` can be tested cleanly and hermetically using `tempfile.TemporaryDirectory()`, executing real bash invocations and verifying output `README.md` contents without violating the "Forbidden Test Workspace Creation" rule.
4. **Premise 4 (Three Distinct Tiers)**: The architecture requires three clean tiers:
   - Tier 1: Upstream Spec Core Compiler (`DEAP01-spec-core`): must not contain customer domain clone commands or inline monkeypatch scripts.
   - Tier 2: Domain Distribution Template (`DEAP-*`): must declare `DOMAIN_DISTRIBUTION_TEMPLATE`, clean landing zones, and provide customer onboarding clone command pointing to itself.
   - Tier 3: Customer Application Workspace (`uav-*`): must declare `DOWNSTREAM_CUSTOMER_PROJECT`, contain zero circular clone commands, and provide local workspace commands.
5. **Deduction / Conclusion**: A dedicated test suite (`tests/test_readme_scaffolding.py`) following the `test_domain_url_synthesis.py` pattern is the optimal mechanism to verify upstream README clean-up, distinct template vs customer README generation, circular clone prevention, and universal shell code fence executability via `bash -n`.

---

## 3. Caveats

- **No live modifications made**: In accordance with the explorer role, this investigation was strictly read-only; no code files in `scripts/` or `tests/` were modified.
- **GitLab CI verification**: Verification of remote GitLab / GitHub issue synchronization was evaluated via script inspection and unit test execution, without invoking network mutations.
- **No caveats** regarding the functionality of `verify_downstream_baseline.py` or existing test harnesses.

---

## 4. Conclusion

The verification facilities in `DEAP01-spec-core` are mature for SysML compilation, KaTeX math, Mermaid diagrams, and domain baseline conformance, but need a dedicated test suite for README scaffolding and shell code fence hygiene.

The proposed test suite `tests/test_readme_scaffolding.py` will implement 4 test suites with 9 concrete tests:
1. `TestUpstreamCompilerReadme`:
   - `test_upstream_readme_no_inline_python_scripts`
   - `test_upstream_readme_no_domain_clone_commands`
   - `test_upstream_readme_compiler_focus_commands`
   - `test_upstream_readme_repository_classification`
2. `TestDomainDistributionTemplateScaffolding`:
   - `test_domain_template_readme_role_declaration`
   - `test_domain_template_readme_clean_landing_zone_guidance`
   - `test_domain_template_readme_customer_onboarding_clone_command`
3. `TestCustomerWorkspaceScaffolding`:
   - `test_customer_workspace_readme_role_declaration`
   - `test_customer_workspace_readme_no_circular_clone`
   - `test_customer_workspace_readme_local_project_commands`
4. `TestShellCodeFenceHygiene`:
   - `test_shell_code_fences_parse_cleanly` (validates every shell block with `bash -n`)
   - `test_shell_comments_zero_unescaped_parentheses` (asserts zero unescaped `(` or `)`)
   - `test_shell_commands_zero_unquoted_angle_brackets` (asserts zero unquoted `<placeholder>`)

---

## 5. Verification Method

To independently verify these findings and execute the baseline tests:

1. **Verify Current Baseline Passes**:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected result*: Exit code 0, all 24+ checks pass.

2. **Run Current Test Suite**:
   ```bash
   python3 -m unittest discover tests
   ```
   *Expected result*: 23 tests pass in ~6.5 seconds with exit code 0.

3. **Verify Upstream Section 5.4 & Section 5.3 Contamination in Current Code**:
   ```bash
   grep -n "python3 -c" README.md
   grep -n "DEAP-uas-infrastructure-safety" README.md
   ```
   *Expected result*: Matches found at lines 253, 262, 310, 348 confirming presence of target purge content.

4. **Verify Report and Briefing Artifacts**:
   - Inspect `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/report.md`
   - Inspect `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_3/BRIEFING.md`
