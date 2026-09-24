# Project: Two-Tier Installation and Propagation Architecture Alignment

## Architecture
Two-tier architecture alignment separating upstream compiler tooling from downstream customer project onboarding:
- Tier 1: Upstream Compiler (DEAP01-spec-core) -> Domain Distribution Template (DEAP-*)
- Tier 2: Domain Distribution Template (DEAP-*) -> Customer Application Workspace (uav-*)

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | R1: Two-Tier Documentation | Document Tier 1 vs Tier 2 boundaries in README.md | M1 | ORIGINAL_REQUEST § R1 |
| 2 | R2: Parameterized Domain Installer | Embed domain remote URL in downstream README and self-contained onboarding command | M2 | ORIGINAL_REQUEST § R2 |
| 3 | R3: Robust Model & Schema Copying | Copy schemas from domain repo to target customer project even if schema/ exists | M3 | ORIGINAL_REQUEST § R3 |
| 4 | Verification & Conformance | python3 scripts/verify_downstream_baseline.py --no-domain, shell syntax check | M4 | ORIGINAL_REQUEST § Acceptance Criteria |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: R1 Two-Tier README | README.md | none | IN_PROGRESS |
| 2 | M2: R2 Parameterized Installer | scripts/install_pipeline.sh | none | IN_PROGRESS |
| 3 | M3: R3 Robust Schema Copying | scripts/install_pipeline.sh | none | IN_PROGRESS |
| 4 | M4: Baseline Verification | scripts/verify_downstream_baseline.py | M1, M2, M3 | PLANNED |

## Interface Contracts
### scripts/install_pipeline.sh ↔ downstream README.md
- Embedding domain remote URL in downstream README.md onboarding command:
  `git clone <domain-repo-remote-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
  No unquoted angle-brackets, zero unescaped parentheses in comments.
### scripts/install_pipeline.sh ↔ TARGET_DIR/schema
- When TARGET_DIR/schema exists, ensure INSTALLER_ROOT/schema contents are copied into TARGET_DIR/schema/ without skipping.
