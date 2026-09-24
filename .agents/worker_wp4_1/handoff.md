# Handoff Report: Work Package 4 - Regression Test Suite for Domain Template URL Synthesis and CLI Parameter Support

## 1. Observation
- Target script: `scripts/install_pipeline.sh` (lines 38-39, 144-167, 643-665).
  - CLI flags: `--domain-url URL` (explicit domain template remote URL) and `--domain-name NAME` (domain template/project name).
  - `--help` documentation at lines 38-39:
    ```
          --domain-url URL       Explicit remote URL for upstream domain template repository
          --domain-name NAME     Domain template or project name (e.g. 'DEAP-uas-infrastructure-safety')
    ```
  - Missing-arg handling at lines 145-148 and 157-160 emits:
    `Error: --domain-url requires a URL argument.`
    `Error: --domain-name requires a name argument.`
  - Domain remote URL synthesis logic at lines 643-665:
    ```bash
      # Resolve domain repository git remote URL for customer onboarding instructions
      DOMAIN_REMOTE_URL=""
      if [ -n "$DOMAIN_URL" ]; then
        DOMAIN_REMOTE_URL="$DOMAIN_URL"
      elif [ -n "$REMOTE_URL" ] && ! echo "$REMOTE_URL" | grep -q "DEAP01-spec-core"; then
        DOMAIN_REMOTE_URL="$REMOTE_URL"
      elif [ -n "$DETECTED_SERVER_URL" ] && [ -n "$DETECTED_NAMESPACE" ] && [ -n "$DETECTED_PROJECT" ] && [ "$DETECTED_PROJECT" != "DEAP01-spec-core" ]; then
        DOMAIN_REMOTE_URL="${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${DETECTED_PROJECT}.git"
      elif [ -n "$DETECTED_SERVER_URL" ] && [ -n "$DETECTED_NAMESPACE" ] && [ -n "$DOMAIN_PROJECT_NAME" ] && [ "$DOMAIN_PROJECT_NAME" != "Downstream Cyber-Physical Infrastructure Safety Project" ]; then
        CLEAN_NAME=$(echo "$DOMAIN_PROJECT_NAME" | tr ' ' '-')
        DOMAIN_REMOTE_URL="${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${CLEAN_NAME}.git"
      elif [ ! -e "$INSTALLER_ROOT/.pipeline/upstream" ]; then
        INSTALLER_REMOTE=$(git -C "$INSTALLER_ROOT" remote get-url origin 2>/dev/null || git -C "$INSTALLER_ROOT" config --get remote.origin.url 2>/dev/null || true)
        if [ -n "$INSTALLER_REMOTE" ] && ! echo "$INSTALLER_REMOTE" | grep -q "DEAP01-spec-core"; then
          DOMAIN_REMOTE_URL="$INSTALLER_REMOTE"
        fi
      fi

      if [ -z "$DOMAIN_REMOTE_URL" ]; then
        CLEAN_NAME=$(echo "${DOMAIN_PROJECT_NAME:-downstream-project}" | tr ' ' '-')
        DOMAIN_REMOTE_URL="https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git"
      fi
    ```
  - Customer project onboarding instruction template in `README.md` at lines 704-706:
    ```bash
    git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
    ```
- Created `tests/test_domain_url_synthesis.py` (251 lines) implementing 9 test methods:
  1. `test_help_shows_domain_options`
  2. `test_domain_url_missing_arg_fails`
  3. `test_domain_name_missing_arg_fails`
  4. `test_domain_remote_url_with_gitlab_provider_defaults_to_github`
  5. `test_explicit_domain_url_flag`
  6. `test_explicit_domain_url_equals_syntax`
  7. `test_target_origin_remote_preserved`
  8. `test_explicit_domain_name_equals_syntax`
  9. `test_domain_url_precedence_over_domain_name`
- Tool commands and results:
  - Command: `python3 -m unittest tests/test_domain_url_synthesis.py -v`
    Result:
    ```
    test_domain_name_missing_arg_fails (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_domain_remote_url_with_gitlab_provider_defaults_to_github (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_domain_url_missing_arg_fails (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_domain_url_precedence_over_domain_name (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_explicit_domain_name_equals_syntax (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_explicit_domain_url_equals_syntax (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_explicit_domain_url_flag (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_help_shows_domain_options (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok
    test_target_origin_remote_preserved (tests.test_domain_url_synthesis.TestDomainUrlSynthesis) ... ok

    Ran 9 tests in 6.460s
    OK
    ```
  - Command: `python3 -m unittest discover tests -v`
    Result: `Ran 23 tests in 6.359s` -> `OK`.
  - Command: `python3 scripts/verify_downstream_baseline.py --no-domain`
    Result: All 30 checks passed cleanly (`Success: Build and test suite execution passed for '/Users/perkunas/jail/DEAP01-spec-core'. Conformance gate verified.`).

## 2. Logic Chain
1. Per Issue #363, `scripts/install_pipeline.sh` previously synthesized invalid `gitlab.com` URLs for upstream domain repositories when `--provider gitlab` was set for downstream customer workspaces.
2. The implementation decoupled the provider setting (`$PROVIDER`) from domain remote synthesis, added explicit CLI options `--domain-url` and `--domain-name`, and preserved existing git remotes when present.
3. To prevent regression, `tests/test_domain_url_synthesis.py` exercises:
   - CLI help documentation verification.
   - CLI option validation (negative error cases for missing arguments and flags passed as argument values).
   - Provider decoupling: `--provider gitlab` with `--domain-name DEAP-uas-infrastructure-safety` produces `https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git` and contains zero `gitlab.com` occurrences in domain URLs or clone commands.
   - Explicit parameter passing: `--domain-url <URL>` and `--domain-url=<URL>` embed the exact target URL into the onboarding command in `README.md`.
   - Existing repository remote preservation: initializing a repo with `origin` set to `https://github.com/org/DEAP-uas-infrastructure-safety.git` ensures that exact origin is retained.
   - Precedence: `--domain-url` takes precedence over `--domain-name`.
4. All test runs operate strictly within `tempfile.TemporaryDirectory` contexts, guaranteeing isolation and preventing test artifact leakage into the workspace.
5. Verification via `unittest` and `verify_downstream_baseline.py` confirms 100% pass rate with zero errors or warnings.

## 3. Caveats
- No implementation bugs were discovered in `scripts/install_pipeline.sh`; all 9 regression tests passed immediately on the existing implementation.
- Tests invoke `bash scripts/install_pipeline.sh` via `subprocess.run`; environments running tests must have `bash` and `git` available (standard on all supported Unix/Linux/macOS platforms).

## 4. Conclusion
Work Package 4 is complete. The test suite `tests/test_domain_url_synthesis.py` thoroughly covers all requirements of Issue #363, verifies CLI parameter handling, provider decoupling, equals syntax, and origin remote preservation. All tests execute cleanly and baseline conformance remains 100% green.

## 5. Verification Method
To independently verify:
1. Run the new regression test suite:
   ```bash
   python3 -m unittest tests/test_domain_url_synthesis.py -v
   ```
   Expected: 9 tests run and pass cleanly with `OK`.
2. Run the complete repository test suite:
   ```bash
   python3 -m unittest discover tests -v
   ```
   Expected: 23 tests run and pass cleanly with `OK`.
3. Run the baseline conformance gate:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   Expected: All 30 checks pass with exit code 0.
