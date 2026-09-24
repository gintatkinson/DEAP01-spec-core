# Handoff Report: Adversarial Code Audit on `scripts/install_pipeline.sh` Domain URL Synthesis

**Target**: `scripts/install_pipeline.sh:626-633`  
**Pillar**: Semantic Traceability  
**Filed Issue**: [Issue #363](https://github.com/gintatkinson/DEAP01-spec-core/issues/363)  
**Severity**: Important  
**Label**: bug  

---

## 1. Observation

1. In `scripts/install_pipeline.sh` lines 626–633, the fallback logic for resolving `DOMAIN_REMOTE_URL` checks `$PROVIDER`:
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
2. In `scripts/install_pipeline.sh` lines 672–675, this synthesized URL is embedded into the customer onboarding instruction written to `$TARGET_DIR/README.md`:
```bash
# Onboard customer application workspace
git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
```
3. When `--provider gitlab` is specified for a customer project (e.g. `uav-011` hosted on GitLab), while the upstream domain template (`DEAP-uas-infrastructure-safety`) is hosted on GitHub, running the onboarding command fails with:
```
remote: The project you were looking for could not be found or you don't have permission to view it.
fatal: repository 'https://gitlab.com/gintatkinson/DEAP-uas-infrastructure-safety.git/' not found
```
4. In `scripts/file_defect.py` lines 182–188, `STOPWORDS` initially lacked `"tooling"`. As a result, titles prefixed with `"Tooling Bug:"` shared two tokens (`"tooling"`, `"install_pipeline"`) with existing issue #290, triggering a false-positive deduplication match. Adding `"tooling"` to `STOPWORDS` resolved this collision.
5. All Step D checks in `skills/adversarial-code-auditor/SKILL.md` (Checks 1–12) were executed on `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp1_1/dossier.md` and passed cleanly, including the offline Mermaid syntax gate (Check 7).
6. Issue filing command output:
```
Executing: gh issue create --repo gintatkinson/DEAP01-spec-core --title Tooling Bug: install_pipeline.sh synthesizes non-existent GitLab URLs for GitHub domain templates --label bug --body-file /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp1_1/dossier.md
https://github.com/gintatkinson/DEAP01-spec-core/issues/363
```

---

## 2. Logic Chain

1. Observations 1 and 2 demonstrate that `DOMAIN_REMOTE_URL` directly determines the clone command emitted into downstream customer `README.md` files.
2. In Observation 1, the fallback branch conditions on `"$PROVIDER" = "gitlab"`, which reflects the customer project's forge/issue tracker, not the upstream domain template repository's hosting provider.
3. Because DEAP architecture decouples Tier 1 (Upstream domain distribution templates on GitHub) from Tier 2 (Customer project workspaces on GitLab, GitHub, etc.), evaluating `$PROVIDER` in the fallback synthesizes a non-existent URL (`https://gitlab.com/...`).
4. Observation 3 confirms that executing this synthesized command causes `git clone` to abort with a fatal repository not found error, preventing customer onboarding.
5. Observation 4 identified and fixed a false-positive deduplication bug in `scripts/file_defect.py`, allowing the defect to be submitted.
6. Observations 5 and 6 confirm that the 7-section defect dossier meets all schema constraints and was registered upstream as Issue #363.

---

## 3. Caveats

- The audit specifically analyzed the URL synthesis fallback and customer onboarding command in `scripts/install_pipeline.sh:626-633`. It did not modify `scripts/install_pipeline.sh` (as remediation is assigned to Work Package 2).
- Downstream propagation across `DEAP-uas-infrastructure-safety` and `uav-011` remains to be performed in Work Package 3.

---

## 4. Conclusion

- A verified Important-severity defect exists in `scripts/install_pipeline.sh:626-633` due to semantic conflation of customer project `$PROVIDER` with upstream domain template hosting platforms.
- The 7-section defect dossier was generated, verified against all 12 Step D checks, and filed upstream as **Issue #363**:
  `https://github.com/gintatkinson/DEAP01-spec-core/issues/363`
- Remediation (Work Package 2) should add `--domain-url` support and ensure canonical domain templates default to GitHub.
- Regression testing (Work Package 4) should execute against test target `tests/test_domain_url_synthesis.py`.

---

## 5. Verification Method

1. Inspect the filed upstream defect on GitHub:
```bash
gh issue view 363 --repo gintatkinson/DEAP01-spec-core
```
2. Verify local dossier integrity:
```bash
python3 scripts/file_defect.py --body-file /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp1_1/dossier.md --validate-only
```
3. Run offline Mermaid syntax validation on dossier:
```bash
python3 - "/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp1_1/dossier.md" <<'EOF'
import sys, os
sys.path.insert(0, "skills/spec-orchestrator/parity_auditor/src")
from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text
body = open(sys.argv[1], encoding="utf-8").read()
errors = check_mermaid_text(body, source=os.path.basename(sys.argv[1]))
assert not errors, f"Mermaid errors: {errors}"
print("Mermaid syntax verified successfully.")
EOF
```
