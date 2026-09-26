# Handoff Report: Cluster B Implementer (R3: Polyrepo Propagation Gate #372)

## 1. Observation
- **Issue Reference**: Issue #372: "Missing cross-repository integration tests and automated propagation gates".
- **Target File Authored**: `tests/test_polyrepo_propagation_gate.py` (720 lines, 11 tests across 4 test classes).
- **Execution Output (Suite)**:
  ```
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoGitLabProvisioning::test_downstream_baseline_fail_closed_when_rules_bundle_missing PASSED [  9%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoGitLabProvisioning::test_downstream_baseline_readiness_gitlab_mode PASSED [ 18%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoGitLabProvisioning::test_provision_downstream_gitlab_sandbox_structure PASSED [ 27%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDomainUrlPreservation::test_gitlab_sandbox_with_custom_domain_forge_url PASSED [ 36%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDomainUrlPreservation::test_gitlab_sandbox_with_domain_name_preserves_github_host PASSED [ 45%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDomainUrlPreservation::test_gitlab_sandbox_with_github_domain_url_preserves_remote PASSED [ 54%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDualProviderIssueCreation::test_create_issue_gitlab_idempotency_prevents_duplicate PASSED [ 63%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDualProviderIssueCreation::test_create_issue_gitlab_link_expansion_uses_dash_blob PASSED [ 72%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoDualProviderIssueCreation::test_create_issue_gitlab_provider_dispatch PASSED [ 81%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoRolloutGateContracts::test_create_issue_script_polyrepo_contract_invariants PASSED [ 90%]
  tests/test_polyrepo_propagation_gate.py::TestPolyrepoRolloutGateContracts::test_installer_polyrepo_contract_invariants PASSED [100%]

  ======================== 11 passed in 337.42s (0:05:37) ========================
  ```
- **Regression Execution Output**:
  ```
  tests/test_domain_url_synthesis.py .........                             [ 22%]
  tests/test_create_issue_dual_provider.py ............................... [100%]

  ============================= 40 passed in 29.65s ==============================
  ```
- **Zero In-Tree Modifications Outside Exclusive Boundary**: Git status confirms only `tests/test_polyrepo_propagation_gate.py` and files inside `.agents/worker_cluster_b/` were touched.

## 2. Logic Chain
1. **Downstream GitLab Sandbox Provisioning (`TestPolyrepoGitLabProvisioning`)**:
   - `scripts/install_pipeline.sh` provisions downstream sandboxes by copying core pipeline tools (`scripts/verify_downstream_baseline.py`, `create_issue.sh`, `reconcile_backlog.py`), safety fixtures (`tests/fixtures/safety/`), `.gitignore`, and compiling `.pipeline/ACTIVE_RULES_BUNDLE.md`.
   - The test asserts that upstream compiler sentinel directory `.pipeline/upstream/` is rigorously excluded from downstream sandboxes to preserve the downstream workspace boundary.
   - The test verifies that `scripts/verify_downstream_baseline.py` executes cleanly in the provisioned repository and fails closed when mandatory governance entrypoints (`rules/sysml-ssot-completeness.md`) are deleted.

2. **Domain URL Preservation & Host Decoupling (`TestPolyrepoDomainUrlPreservation`)**:
   - Issue #363 / #372 requires that when provisioning a GitLab customer workspace with an upstream domain template (`DEAP-*`), the installer must decouple the customer project git host from the upstream template host (GitHub).
   - The test verifies that:
     * Passing `--domain-url https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git` embeds the GitHub clone command in the domain README.
     * Passing `--domain-name DEAP-uas-infrastructure-safety` resolves to GitHub domain template.
     * Passing an arbitrary domain forge URL (`https://forge.internal.defense.gov/...`) is preserved verbatim.
     * Zero invalid synthesized `gitlab.com` routes are generated for GitHub domain repositories.

3. **Downstream Dual-Provider Issue Creation (`TestPolyrepoDualProviderIssueCreation`)**:
   - Provisions a live sandbox, installs mock `glab` and `gh` CLIs to isolate network calls, creates authentic schema-derived SysML v2 AST, Epic, and Feature specifications satisfying all 23 linter gates.
   - Asserts `create_issue.sh` executes `glab` CLI with `--description-file` (preventing `ARG_MAX` buffer overflow per #374).
   - Asserts exact title column indexing (column 2 for GitLab) prevents duplicate issue creation per #373.
   - Asserts relative markdown links are expanded into GitLab `/-/blob/<branch>/...` syntax rather than GitHub `/blob/...` syntax per #244.

4. **Polyrepo Rollout Gate Contracts (`TestPolyrepoRolloutGateContracts`)**:
   - Statically verifies `scripts/install_pipeline.sh` option flags, active rule bundling, upstream marker purging, and OS metadata cleanup.
   - Statically verifies `create_issue.sh` export of `TITLE`, `--description-file` and `--body-file` usage, dual-provider awk column checks (`$2` for GitLab, `$3` for GitHub), and integration with `reconcile_backlog.py` for `/-/blob/` link expansion.

## 3. Caveats
- All test sandboxes use isolated temporary directories via `tempfile.TemporaryDirectory()`. No live API requests are made to GitHub or GitLab during test execution; mock binary shims intercept `glab` and `gh` commands and verify payloads.
- Testing baseline verification in downstream sandboxes requires setting `DEAP_REPOSITORY_TYPE="UPSTREAM_SPEC_CORE_COMPILER"` if the sandbox does not contain an enterprise architecture specification corpus (`docs/conops/`, `docs/interfaces/`, `docs/safety/`), in accordance with fail-closed Gate 30 (`ArchitectureViewpointValidator`, issue #365).

## 4. Conclusion
- Issue #372 is completely remediated by `tests/test_polyrepo_propagation_gate.py`.
- Automated cross-repository integration tests and polyrepo rollout gates are established and verified.
- 100% test pass rate achieved on the new test suite (11/11 PASSED).
- Zero regressions across existing dual-provider test suites (40/40 PASSED).

## 5. Verification Method
Execute the following verification commands:
```bash
# 1. Verify polyrepo propagation gate suite
python3 -m pytest tests/test_polyrepo_propagation_gate.py -v

# 2. Verify regression safety across dual-provider test suites
python3 -m pytest tests/test_domain_url_synthesis.py tests/test_create_issue_dual_provider.py
```
Expected result: 11 passed in `test_polyrepo_propagation_gate.py`, 40 passed in existing suites. Zero failures.
