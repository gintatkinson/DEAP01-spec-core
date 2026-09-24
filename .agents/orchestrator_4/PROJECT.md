# Project: Three-Tier Onboarding & README Template Overhaul

## Architecture
- Three-tier repository architecture:
  1. Tier 1: Upstream Spec Core Compiler (`DEAP01-spec-core`)
  2. Tier 2: Domain Distribution Template (`DEAP-*`, e.g. `DEAP-uas-infrastructure-safety`)
  3. Tier 3: Customer Application Workspace (`uav-*`, e.g. `uav-011`)
- Artifact propagation flow:
  `DEAP01-spec-core` -> `scripts/install_pipeline.sh "<path-to-domain-template>"` -> `DEAP-*` -> `git clone "<domain-url>" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline` -> `uav-*`

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Purge Section 5.4 | Remove 90-line Python monkeypatch script & cp loops from README.md | M1 | Survey (explorer_r4_1) / ORIGINAL_REQUEST § R1 (DONE) |
| 2 | Compiler-Centric Focus | Document compiler execution, tests, and clean propagation commands | M1 | Survey (explorer_r4_1) / ORIGINAL_REQUEST § R1 (DONE) |
| 3 | Remove Conflated Domain Content | Remove hardcoded domain clone commands from compiler quickstart | M1 | Survey (explorer_r4_1) / ORIGINAL_REQUEST § R1 (DONE) |
| 4 | Dynamic Tier Detection | Detect DOMAIN_DISTRIBUTION_TEMPLATE vs DOWNSTREAM_CUSTOMER_PROJECT in install_pipeline.sh | M2 | Survey (explorer_r4_2) / ORIGINAL_REQUEST § R2, R3 (DONE) |
| 5 | Domain Template README Scaffolding | Generate README declaring DOMAIN_DISTRIBUTION_TEMPLATE & single-line customer clone command | M2 | Survey (explorer_r4_2) / ORIGINAL_REQUEST § R2 (DONE) |
| 6 | Customer Workspace README Scaffolding | Generate README declaring DOWNSTREAM_CUSTOMER_PROJECT, no circular clone, project commands | M2 | Survey (explorer_r4_2) / ORIGINAL_REQUEST § R3 (DONE) |
| 7 | Verification & Conformance Gates | Baseline verification, pure shell code fences, zero unescaped parens/angle brackets | M3 | Survey (explorer_r4_3) / ORIGINAL_REQUEST § Acceptance Criteria (DONE) |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Upstream Compiler README (R1) | Clean README.md in DEAP01-spec-core | None | DONE |
| 2 | Installer Scaffolding (R2, R3) | scripts/install_pipeline.sh dynamic role detection and template scaffolding | M1 | DONE |
| 3 | Verification & Baseline Gates | Conformance tests and verify_downstream_baseline.py --no-domain | M1, M2 | DONE |

## Code Layout
- `README.md`: Upstream compiler top-level documentation
- `scripts/install_pipeline.sh`: Installer and template scaffolding engine
- `tests/test_readme_scaffolding.py`: Unit and regression test suite
- `scripts/verify_downstream_baseline.py`: Baseline validator for downstream environments
