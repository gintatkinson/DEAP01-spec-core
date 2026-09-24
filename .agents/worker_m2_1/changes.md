# Changes Report - Milestone 2 (R2 and R3)

**Agent:** `worker_m2_1`  
**Role:** Implementer / QA  
**Target File:** `scripts/install_pipeline.sh`  
**Date:** 2026-09-21  

---

## 1. Summary of Changes

Milestone 2 has been implemented exclusively in `scripts/install_pipeline.sh` to resolve the repository tier confusion, dynamic role detection, distinct README scaffolding, and regeneration trigger condition:

1. **CLI Argument Parsing & Validation (`-r, --role ROLE`)**:
   - Added `CLI_ROLE` variable and `-r, --role ROLE` option documentation to `show_help()`.
   - Added parsing cases `-r|--role`, `--role=*`, and `-r=*`.
   - Validates role against accepted values:
     - Domain Template: `domain-template`, `domain_template`, `domain`, `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - Customer Workspace: `customer-project`, `customer_project`, `customer`, `downstream_customer_project`, `DOWNSTREAM_CUSTOMER_PROJECT`, `downstream-application-workspace`.
   - Rejects invalid roles with exit code 1 and descriptive stderr message.

2. **Dynamic Repository Role Auto-Detection**:
   - When `--role` is omitted, auto-detects `TARGET_ROLE` cleanly based on priority:
     - Target directory basename (`$DIR_BASE` matches `DEAP-*`).
     - Git remote URL (`$REMOTE_URL` matches `*DEAP-*`).
     - Detected project name (`$DETECTED_PROJECT` matches `DEAP-*`).
     - Explicit `--domain-name` starting with `DEAP-*`.
     - Explicit `--domain-url` supplied.
     -> Resolves to `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - Otherwise (or if `$DIR_BASE` matches `uav-*` or `$DETECTED_PROJECT` matches `uav-*`):
     -> Resolves to `DOWNSTREAM_CUSTOMER_PROJECT`.

3. **Distinct README Scaffolding by Role**:
   - **`DOMAIN_DISTRIBUTION_TEMPLATE` (DEAP-*)**:
     - Declares `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\``.
     - Section 1.1 documents the Clean Landing Zone Invariant in `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`.
     - Section 1.2 declares MATLAB / Simulink / Stateflow / Embedded Coder toolchain context.
     - Section 3.1 documents the single-line customer onboarding clone command:
       ```bash
       git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
       ```
     - Section 3.2 documents in-place template tooling update (`bash scripts/install_pipeline.sh .`).
     - Preserves Section 4 Operator Prompt Catalog and Section 5 Quality Gates.
   - **`DOWNSTREAM_CUSTOMER_PROJECT` (uav-*)**:
     - Declares `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``.
     - Section 1.1 documents Customer Application Workspace Scope (concrete flight code, ROS2 nodes, PX4 flight modules).
     - Section 1.2 declares MATLAB / Simulink / Stateflow / Embedded Coder toolchain context.
     - Section 3.1 documents downstream baseline verification:
       ```bash
       python3 scripts/verify_downstream_baseline.py --no-domain
       ```
       and Level 0 OEM Ground Truth Ingestion (Step 0.0) + SysML compilation gate:
       ```bash
       python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema "schema/extracted/" --format markdown --out "schema/model.sysml"
       python3 scripts/compile_sysml.py --compile
       ```
     - Section 3.2 documents in-place tooling update:
       ```bash
       bash scripts/install_pipeline.sh .
       ```
     - Eliminates circular clone instructions completely.
     - Preserves Section 4 Operator Prompt Catalog and Section 5 Quality Gates.

4. **Role-Aware README Regeneration Trigger Condition**:
   - Replaced fragile lines 517–520 with role-aware `SHOULD_SCAFFOLD_README` evaluation:
     - Re-scaffolds if README missing or contains GitLab onboarding boilerplate.
     - Re-scaffolds if Operator Prompt Catalog is missing.
     - If role is `DOMAIN_DISTRIBUTION_TEMPLATE`: re-scaffolds if missing `DOMAIN_DISTRIBUTION_TEMPLATE` or customer onboarding instructions.
     - If role is `DOWNSTREAM_CUSTOMER_PROJECT`: re-scaffolds if it contains the legacy circular `git clone.*\.tmp-pipeline` command, or is missing `DOWNSTREAM_CUSTOMER_PROJECT` or `Project Lifecycle & Tooling Maintenance`.
     - Idempotent: Does NOT overwrite compliant READMEs for either role.

5. **Code Fence & Shell Syntax Hygiene**:
   - All comments inside shell code blocks contain pure valid shell comments with zero unescaped parentheses.
   - All shell blocks verified executable with `bash -n`.
   - Zero unquoted angle-bracket placeholders.

---

## 2. Verification Summary

- `bash -n scripts/install_pipeline.sh`: Syntax check passed (exit 0).
- `python3 -m unittest discover tests`: All 23 tests (including `test_domain_url_synthesis.py`) passed cleanly.
- `python3 scripts/verify_downstream_baseline.py --no-domain`: All 30 checks passed cleanly.
- Hermetic test suite in `/tmp`:
  - `test_role_domain_template_cli`: PASS
  - `test_role_customer_project_cli`: PASS
  - `test_role_autodetect_deap`: PASS
  - `test_role_autodetect_uav`: PASS
  - `test_idempotence_and_preservation`: PASS
  - `test_shell_fence_hygiene`: PASS
  - `test_old_circular_readme_upgrade`: PASS
  - `test_cli_role_error_handling`: PASS
