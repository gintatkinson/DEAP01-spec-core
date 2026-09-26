# Handoff Report: Phase 1 Defect Triage & Empirical Evidence Audit

**Agent:** Phase 1 Triage & Evidence Explorer  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/`  
**Target Milestone:** Phase 1 (R1: Comprehensive Triage & Evidence Audit)  
**Date:** 2026-09-26  

---

## 1. Observation

Direct empirical observations from inspecting the 17 tracked defect issues (#378, #377, #376, #375, #374, #373, #372, #368, #366, #365, #364, #363, #362, #361, #360, #349, #286), git commits between `d0e1bf0` and `dd7638c` (and foundational tooling commits `90fe8fa`, `14932ff`, `ce82ef4`, `4eedb5b`), active source files, and unit tests:

1. **Issue #368 (Governance Rules Bundle)**:
   - Commit `14932ff` ("feat(pipeline): bundle active governance rules into ACTIVE_RULES_BUNDLE.md (refs #368)") added compiler logic to `scripts/install_pipeline.sh` (lines 483-530) compiling `.pipeline/ACTIVE_RULES_BUNDLE.md` directly from `rules/*.md`.
   - Commit `080fc49` updated upgrade detection logic in `scripts/install_pipeline.sh` (lines 697, 707) and README scaffolding (lines 894, 930, 965, 1011, 1461, 1495, 1524).
   - Commit `90fe8fa` committed `.pipeline/ACTIVE_RULES_BUNDLE.md` (1849 lines).
   - Command `python3 -m pytest tests/test_readme_scaffolding.py` executed: **31 passed in 31.67s** (exit code 0).

2. **Issue #363 (Domain Template URLs)**:
   - Commits `4eedb5b`, `196512d`, and `ce82ef4` added `--domain-url` and `--domain-name` CLI flags to `scripts/install_pipeline.sh` (lines 84-128, 324-335, 626-640), decoupling customer workspace `--provider` (GitHub/GitLab) from upstream domain template repository URLs (`https://github.com/gintatkinson/DEAP-*.git`).
   - Command `python3 -m pytest tests/test_domain_url_synthesis.py` executed: **9 passed in 8.40s** (exit code 0).

3. **Issue #373 (create_issue.sh Title Indexing)**:
   - Commit `90fe8fa` updated `skills/spec-orchestrator/scripts/create_issue.sh`:
     Line 64 exports `TITLE` (`export TITLE`).
     Line 184 escapes quotes (`ESCAPED_TITLE="${TITLE//\"/\\\"}"`).
     Line 195 indexes column 3 (Title) instead of column 2:
     `EXISTING=$(printf '%s\n' "$GH_LIST_OUT" | awk -F'\t' '$3 == ENVIRON["TITLE"] { print $1; exit }')`
   - Command `python3 -m pytest tests/test_create_issue_dual_provider.py -k test_github_idempotency_avoids_duplicate` executed: **1 passed** (suite total: **31 passed in 20.44s**, exit code 0). Line 773 asserts `self.assertIn('$3 == ENVIRON["TITLE"]', content)`.

4. **Issue #374 (create_issue.sh ARG_MAX Buffer Overflow)**:
   - Commit `90fe8fa` updated `skills/spec-orchestrator/scripts/create_issue.sh`:
     Line 305 invokes `glab issue create $REPO_FLAG --title "$TITLE" --label "$LABEL" --description-file "$TMP_EXPANDED_BODY"` using `--description-file` instead of passing the body inline as `--description "$(...)"`.
     Line 307 invokes `gh issue create $REPO_FLAG --title "$TITLE" --label "$LABEL" --body-file "$TMP_EXPANDED_BODY"`.
   - Command `python3 -m pytest tests/test_create_issue_dual_provider.py` executed: **31 passed in 20.44s** (exit code 0). Line 775 asserts `self.assertIn('--description-file "$TMP_EXPANDED_BODY"', content)`.

5. **Active Issues in Cluster A (#378, #377, #376, #364)**:
   - In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py:94-98, 107-113`, `_has_epistemic_exemption()` still uses regex `EPISTEMIC_EXEMPTION_PATTERN` unconditionally exempting entire markdown lines containing `[TIER-3]`, `(TIER-3)`, or `Declared Assumption`.
   - In `skills/schema-specification-engineering/SKILL.md:1-250`, subagent dispatch instructions do not project a typed parameter dictionary AST from the SysML model.
   - In `factual_grounding_validator.py:2614-2690`, Mermaid sequence diagrams are partially recognized, but statecharts, notes, messages, and non-Mermaid code fences unconditionally bypass numeric provenance checks. Zero tests exist in `tests/test_factual_grounding_validator.py`.

6. **Active Issues in Cluster B (#372)**:
   - `tests/test_polyrepo_propagation_gate.py` does not exist. No automated CI polyrepo propagation gate runs against downstream GitLab repositories.

7. **Active Issues in Cluster C (#375, #366, #365, #362, #361)**:
   - In `scripts/verify_downstream_baseline.py`, Check 17 (lines 2075-2088), Check 20 (line 2494), and Check 23 (line 2855) catch missing directories or models and return exit code 0 ("Success"), masking empty landing zones.
   - In `scripts/verify_downstream_baseline.py:1348-1371`, `_discover_sysml_model_text` reads either `schema/*.sysml` or `.pipeline/schema.sysml` without asserting dual-schema parity.
   - In `architecture_viewpoint_validator.py:1240-1248` and `scripts/verify_downstream_baseline.py:3432`, Gate 30 calls `validate(..., allow_missing_specs=True)` which returns empty findings on missing specs, printing success on an empty architecture corpus.
   - In `README.md:772`, command `python3 -m unittest tests.test_conops_and_mission_intent_validators` fails with `ModuleNotFoundError` because the test was moved to `archive/unit_tests_legacy/`.
   - In `README.md:303-314` and `scripts/install_pipeline.sh:921-933`, the initialization sequence ends at step 5 without directing agents to Section 9 (the Multi-Pipeline Operator Prompt Catalog).

8. **Active Issues in Cluster D (#360, #349, #286)**:
   - In `scripts/compile_sysml.py:3169-3234`, `SysMLCapabilityDef` parsing and Epic generation are absent, and `--forward-sync` lacks a Phase Gate Guard.
   - In `archive/unit_tests_legacy/test_cross_document_diagram_parity.py:49-316` and `archive/unit_tests_legacy/test_check23_factual_grounding_gate.py:77-386`, tests rely on synthetic in-memory string mocks with hardcoded aerospace domain concepts (`SwarmC2GroundStation`, `AlphaPlatform`), violating SSOT and Pure Compiler invariants.

---

## 2. Logic Chain

1. **Identification of Remediated Defects (Issues #374, #373, #368, #363)**:
   - Based on Observations 1 through 4, commits `14932ff`, `080fc49`, `4eedb5b`, `ce82ef4`, and `90fe8fa` introduced exact code fixes for rules bundling (#368), domain template URLs (#363), title column indexing in duplicate detection (#373), and `--description-file` payload streaming (#374).
   - Dedicated unit test suites (`tests/test_readme_scaffolding.py`, `tests/test_domain_url_synthesis.py`, `tests/test_create_issue_dual_provider.py`) verify each of these 4 fixes with 100% pass rates.
   - Therefore, these 4 issues are fully **REMEDIATED** and require only verification evidence comments and `status:fixed-resolved` label transitions on GitHub.

2. **Categorization of Active Defects (13 Issues)**:
   - Based on Observation 5, Issues #378, #377, #376, and #364 share a common failure mode: negative-string regex heuristics and bypasses in `factual_grounding_validator.py` and `skills/schema-specification-engineering/SKILL.md`. They belong to **Cluster A (R2: AST Grounding & Anti-Regex Hardening)**.
   - Based on Observation 6, Issue #372 addresses the lack of automated polyrepo integration tests and rollout gates between GitHub and GitLab. It belongs to **Cluster B (R3: Dual-Provider Tooling & Installer Hardening)**.
   - Based on Observation 7, Issues #375, #366, #365, #362, and #361 share the root cause of permissive validator defaults, missing test harnesses, and missing discovery links in baseline verification. They belong to **Cluster C (R4: Baseline Gate Masking & SSOT Parity)**.
   - Based on Observation 8, Issues #360, #349, and #286 share the root cause of synthetic mocks, hardcoded domain models, and premature forward sync. They belong to **Cluster D (R5: Synthetic Mock Elimination)**.

---

## 3. Caveats

- GitHub issue transition commands (`gh issue comment`, `gh issue edit --add-label`) were formulated with exact empirical evidence in `triage_report.md` but not executed in this turn to honor the Read-Only Investigation role lock.
- Issue #376 was partially patched in commit `8d21928` to match parentheses, but its underlying negative-regex evasion vulnerability is subsumed by #378.
- Issue #364 was partially addressed in commit `8d21928` for Mermaid sequence diagrams, but statecharts and general code fences still bypass numeric checks, and tests in `tests/test_factual_grounding_validator.py` remain to be written.

---

## 4. Conclusion

- **Total Issues Audited:** 17
- **Remediated Issues:** 4 (#374, #373, #368, #363)
  * Verification evidence formulated and ready for GitHub posting.
  * Status label transition to `status:fixed-resolved` ready.
- **Active Issues:** 13
  * **Cluster A (R2: AST Grounding & Anti-Regex Hardening):** #378, #377, #376, #364
  * **Cluster B (R3: Dual-Provider Tooling & Installer Hardening):** #372
  * **Cluster C (R4: Baseline Gate Masking & SSOT Parity):** #375, #366, #365, #362, #361
  * **Cluster D (R5: Synthetic Mock Elimination):** #360, #349, #286
- The detailed report is saved at `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/triage_report.md`.
- Phase 1 investigation is complete; Phase 2 implementation subagents may proceed according to the execution plan.

---

## 5. Verification Method

To independently reproduce and verify all observations:

1. **Verify #368 (Governance Rules Bundle)**:
   ```bash
   python3 -m pytest tests/test_readme_scaffolding.py
   # Expected: 31 passed in ~31s, exit code 0
   ```
2. **Verify #363 (Domain Template URLs)**:
   ```bash
   python3 -m pytest tests/test_domain_url_synthesis.py
   # Expected: 9 passed in ~8s, exit code 0
   ```
3. **Verify #373 & #374 (Dual-Provider create_issue.sh Contracts)**:
   ```bash
   python3 -m pytest tests/test_create_issue_dual_provider.py
   # Expected: 31 passed in ~20s, exit code 0
   ```
4. **Reproduce Issue #362 (Missing ConOps Test Harness)**:
   ```bash
   python3 -m unittest tests.test_conops_and_mission_intent_validators
   # Expected: ModuleNotFoundError: No module named 'tests.test_conops_and_mission_intent_validators', exit code 1
   ```
5. **Inspect Active Issue Code Sites**:
   ```bash
   # Check 17, 20, 23 silent exits:
   sed -n '2075,2088p; 2493,2495p; 2854,2856p' scripts/verify_downstream_baseline.py
   # Gate 30 allow_missing_specs=True:
   sed -n '3430,3435p' scripts/verify_downstream_baseline.py
   # _has_epistemic_exemption regex bypass:
   sed -n '94,115p' skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py
   ```
