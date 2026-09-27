# Progress: WP-02 Architecture Tier Hierarchy, Heading Normalization & Repository Boundary Hardening

Last visited: 2026-09-27T15:59:15Z
Status: Completed

## Tasks
- [x] Initialized DISPATCH.md, BRIEFING.md, and progress.md
- [x] Executed view_file on skills/spec-orchestrator/SKILL.md as very first step
- [x] Verified .pipeline/constitution.md via direct path read
- [x] Inspected README.md and scripts/install_pipeline.sh targets
- [x] Baseline test verification (31 tests passed in tests/test_readme_scaffolding.py)
- [x] Apply Section 1 normalization and repositioning to README.md
  * Repositioned 1.1 before 1.2 as H3 (`### 1.1 Primary Commercial Toolchain Integration`)
  * Renamed 1.2 to `### 1.2 Three-Tier Architecture & Repository Boundaries: Upstream Compiler vs. Domain Templates vs. Customer Workspaces`
  * Updated ASCII topology diagram and subsections to clean 3-tier hierarchy (Tier 1 Compiler, Tier 2 Domain Templates, Tier 3 Customer Workspaces)
- [x] Apply Section 4 normalization to README.md
  * Updated line 152 to `#### Tier 2: Domain Distribution Template Repository`
  * Updated line 175 to `Tier 3 Customer Application Workspaces`
  * Updated line 177 to `### 4.1 Supported Tier 2 Domain Distribution Templates (Canonical Taxonomies)`
- [x] Apply Section 5.4 normalization and anchor update to README.md
  * Updated text and ASCII diagram to clean three-tier architecture
  * Updated anchor reference to `#41-supported-tier-2-domain-distribution-templates-canonical-taxonomies`
- [x] Apply Section 9.4 boundary invariant and preamble normalization to README.md
  * Added Execution Boundary Invariant admonition restricting Pipeline 2 to DOWNSTREAM_CUSTOMER_PROJECT
  * Removed `(or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` from lines 1105, 1160, 1190
- [x] Apply tier normalization and preamble fixes to scripts/install_pipeline.sh
  * Updated line 882 to `As a **Tier 2 Domain Distribution Template**`
  * Updated line 953 to `As a **Tier 3 Customer Application Workspace**`
  * Updated lines 1430, 1484, 1516 to `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`
- [x] Validated shell script syntax with `bash -n scripts/install_pipeline.sh` (exit code 0)
- [x] Run verification tests:
  * `python3 -m unittest tests/test_readme_scaffolding.py`: 31 tests passed (exit code 0)
  * `python3 scripts/verify_downstream_baseline.py --no-domain`: all 31 checks passed (exit code 0)
- [x] Generated handoff.md and notified orchestrator
