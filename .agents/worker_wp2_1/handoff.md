# Handoff Report: Work Package 2 (Issue #363 Remediation) Implementation

**Author:** Worker WP2 (`.agents/worker_wp2_1`)  
**Target File Exclusively Modified:** `scripts/install_pipeline.sh`  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Primary Commercial Toolchain Integration Context:** `MATLAB / Simulink / Stateflow / Embedded Coder`  

---

## 1. Observation

Direct examination of `scripts/install_pipeline.sh` prior to remediation revealed:

1. **Missing CLI Option for Explicit Domain URL and Domain Name:**
   - Lines 5-13 declared variables:
     ```bash
     TARGET_DIR=""
     PROVIDER="auto"
     GITLAB_URL="https://gitlab.com"
     GITLAB_GROUP=""
     GITHUB_ORG=""
     JIRA_URL="https://your-domain.atlassian.net"
     JIRA_PROJECT=""
     JIRA_EMAIL=""
     ```
     Neither `DOMAIN_URL` nor `DOMAIN_NAME` was declared or parsed.
   - `show_help()` (lines 14-48) had no documentation for `--domain-url <URL>` or `--domain-name <NAME>`.
   - The CLI argument parsing loop (lines 50-156) lacked case blocks for `--domain-url`, `--domain-url=*`, `--domain-name`, or `--domain-name=*`. Passing `--domain-url` or `--domain-name` triggered line 140 `Error: Unknown option: $1` and exited with code 1.

2. **Domain URL Fallback Conflated Customer Provider with Domain Host (Lines 626–633):**
   ```bash
   if [ -z "$DOMAIN_REMOTE_URL" ]; then
     CLEAN_NAME=$(echo "${DOMAIN_PROJECT_NAME:-downstream-project}" | tr ' ' '-')
     if [ "$PROVIDER" = "gitlab" ]; then
       DOMAIN_REMOTE_URL="${GITLAB_URL:-https://gitlab.com}/${GITLAB_GROUP:-your-group}/${CLEAN_NAME}.git"
     else
       DOMAIN_REMOTE_URL="https://github.com/${GITHUB_ORG:-your-org}/${CLEAN_NAME}.git"
     fi
   fi
   ```
   When installing into a downstream customer project whose issue tracker and git host was GitLab (`--provider gitlab`), line 628 caused the fallback to synthesize a non-existent `gitlab.com` URL (e.g. `https://gitlab.com/your-group/DEAP-uas-infrastructure-safety.git` or `https://gitlab.com/gintatkinson/DEAP-uas-infrastructure-safety.git`) rather than pointing to the canonical domain template on GitHub (`https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git`).

3. **No Priority for Explicit `DOMAIN_URL` in `DOMAIN_REMOTE_URL` Resolution:**
   The `DOMAIN_REMOTE_URL` resolution block (lines 610–625) only inspected `REMOTE_URL`, `DETECTED_SERVER_URL`, and `INSTALLER_REMOTE`, with no mechanism for users or CI scripts to provide an explicit `--domain-url` override.

---

## 2. Logic Chain

1. **CLI Parameter Support & Error Handling (R1):**
   - Declared `DOMAIN_URL=""` and `DOMAIN_NAME=""` in variable initialization.
   - Added documentation in `show_help()`:
     ```text
           --domain-url URL       Explicit remote URL for upstream domain template repository
           --domain-name NAME     Domain template or project name (e.g. 'DEAP-uas-infrastructure-safety')
     ```
   - Added `--domain-url`, `--domain-url=*`, `--domain-name`, and `--domain-name=*` to the CLI argument parser:
     ```bash
         --domain-url)
           if [[ -z "$2" || "$2" == -* ]]; then
             echo "Error: --domain-url requires a URL argument." >&2
             exit 1
           fi
           DOMAIN_URL="$2"
           shift 2
           ;;
         --domain-url=*)
           DOMAIN_URL="${1#*=}"
           shift
           ;;
         --domain-name)
           if [[ -z "$2" || "$2" == -* ]]; then
             echo "Error: --domain-name requires a name argument." >&2
             exit 1
           fi
           DOMAIN_NAME="$2"
           shift 2
           ;;
         --domain-name=*)
           DOMAIN_NAME="${1#*=}"
           shift
           ;;
     ```
   - Validated that if `--domain-url` or `--domain-name` is passed without an argument or followed by another flag, it prints an informative error message and exits with code 1.
   - Wired `DOMAIN_NAME` into metadata extraction before fallback checks so that passing `--domain-name` explicitly establishes `DOMAIN_PROJECT_NAME`.

2. **Decoupling Customer Provider from Domain Host (R2):**
   - Upstream domain distribution templates (e.g. `DEAP-uas-infrastructure-safety`) canonicalize on GitHub under `gintatkinson`. The customer project provider (`$PROVIDER`) governs the downstream customer's issue tracking and repo host, NOT the host platform of the upstream domain template.
   - Prioritized `DOMAIN_URL` at the top of the `DOMAIN_REMOTE_URL` resolution block:
     ```bash
       DOMAIN_REMOTE_URL=""
       if [ -n "$DOMAIN_URL" ]; then
         DOMAIN_REMOTE_URL="$DOMAIN_URL"
       elif [ -n "$REMOTE_URL" ] && ! echo "$REMOTE_URL" | grep -q "DEAP01-spec-core"; then
     ```
   - In the fallback logic (lines 626–633), removed the `if [ "$PROVIDER" = "gitlab" ]` check entirely:
     ```bash
       if [ -z "$DOMAIN_REMOTE_URL" ]; then
         CLEAN_NAME=$(echo "${DOMAIN_PROJECT_NAME:-downstream-project}" | tr ' ' '-')
         DOMAIN_REMOTE_URL="https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git"
       fi
     ```
   - This ensures fallback resolves to GitHub under `${GITHUB_ORG:-gintatkinson}` and never synthesizes a non-existent `gitlab.com` URL.

3. **Downstream Onboarding Command Verification (R3):**
   - The onboarding command generated in downstream `README.md`:
     ```bash
     git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
     ```
     now resolves to `https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git` (or the explicit `--domain-url`) even when running with `--provider gitlab`.

---

## 3. Caveats

- **Scope Boundary:** Modifications were strictly confined to `scripts/install_pipeline.sh`. No other repository files were altered.
- **GitLab Provider Scope:** Decoupling `$PROVIDER` from `DOMAIN_REMOTE_URL` applies specifically to the domain template onboarding URL. Downstream tracker label bootstrapping, CI/CD templates (`.gitlab-ci.yml`), and issue creation continue to honor `$PROVIDER=gitlab`.
- **Pure Schema-Driven Compiler Invariant:** Domain concepts are not hardcoded in compiler logic; `GITHUB_ORG` defaults to `gintatkinson` and `CLEAN_NAME` derives dynamically from `DOMAIN_PROJECT_NAME` or `--domain-name`.

---

## 4. Conclusion

Work Package 2 (Issue #363 Remediation) is completely implemented and verified in `scripts/install_pipeline.sh`:
1. Added `--domain-url <URL>` and `--domain-url=*` CLI parameter support with strict validation (missing arg raises error and exits 1).
2. Added `--domain-name <NAME>` and `--domain-name=*` CLI parameter support with strict validation.
3. Updated `show_help()` to document `--domain-url <URL>` and `--domain-name <NAME>`.
4. Prioritized `DOMAIN_URL` at the top of the `DOMAIN_REMOTE_URL` resolution block (`if [ -n "$DOMAIN_URL" ]; then DOMAIN_REMOTE_URL="$DOMAIN_URL"`).
5. Removed `if [ "$PROVIDER" = "gitlab" ]` from fallback logic; fallback resolves deterministically to GitHub (`https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git`).
6. Verified manual test cases in temporary sandbox `/tmp/test_install_domain_url`:
   - `bash scripts/install_pipeline.sh /tmp/test_install_domain_url --provider gitlab --domain-name "DEAP-uas-infrastructure-safety"` produces `git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline` and 0 occurrences of `gitlab.com` in `README.md`.
   - `bash scripts/install_pipeline.sh /tmp/test_install_domain_url --provider gitlab --domain-url https://example.com/custom-domain.git` produces `git clone https://example.com/custom-domain.git ./.tmp-pipeline`.
7. `bash -n scripts/install_pipeline.sh` passes with exit code 0.
8. `python3 scripts/verify_downstream_baseline.py --no-domain` passes all 30 checks cleanly with exit code 0.

---

## 5. Verification Method

To independently verify these changes:

1. **Syntax Check:**
   ```bash
   bash -n scripts/install_pipeline.sh
   ```
   *Expected:* Exit code 0, no output.

2. **Argument Validation Tests:**
   ```bash
   bash scripts/install_pipeline.sh --domain-url
   # Expected: Error: --domain-url requires a URL argument. (Exit code 1)

   bash scripts/install_pipeline.sh --domain-url --provider gitlab
   # Expected: Error: --domain-url requires a URL argument. (Exit code 1)

   bash scripts/install_pipeline.sh --domain-name
   # Expected: Error: --domain-name requires a name argument. (Exit code 1)

   bash scripts/install_pipeline.sh --domain-name --provider gitlab
   # Expected: Error: --domain-name requires a name argument. (Exit code 1)
   ```

3. **Domain Remote URL Decoupling Verification with GitLab Provider:**
   ```bash
   rm -rf /tmp/test_install_domain_url
   bash scripts/install_pipeline.sh /tmp/test_install_domain_url --provider gitlab --domain-name "DEAP-uas-infrastructure-safety"
   grep "git clone" /tmp/test_install_domain_url/README.md
   # Expected output:
   # git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline

   grep "gitlab.com" /tmp/test_install_domain_url/README.md
   # Expected: Exit code 1 (no occurrences found)
   ```

4. **Explicit `--domain-url` Verification:**
   ```bash
   rm -rf /tmp/test_install_domain_url
   bash scripts/install_pipeline.sh /tmp/test_install_domain_url --provider gitlab --domain-url https://example.com/custom-domain.git
   grep "git clone" /tmp/test_install_domain_url/README.md
   # Expected output:
   # git clone https://example.com/custom-domain.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
   ```

5. **Full Baseline Conformance Gate:**
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected:* All 30 checks pass with exit code 0.

6. **Invalidation Conditions:**
   - If running `install_pipeline.sh` with `--provider gitlab` generates a `gitlab.com` URL for a domain template in `README.md`.
   - If `--domain-url` without an argument does not return exit code 1 with an error on stderr.
   - If `--domain-url <URL>` is overridden by downstream heuristics instead of taking priority.
   - If `verify_downstream_baseline.py --no-domain` fails.

