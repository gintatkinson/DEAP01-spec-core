# Implementation Plan: Three-Tier Onboarding & README Template Overhaul

## Goal Description
Overhaul the installation instructions and README templates across all three repository tiers to eliminate broken, contradictory, and circular onboarding instructions:
1. **Upstream Spec Core Compiler (`DEAP01-spec-core`)**: Clean, purpose-driven compiler documentation removing Section 5.4's inline monkeypatching script and manual cp loops, focusing strictly on running/verifying the compiler and propagating tooling to domain templates.
2. **Domain Distribution Templates (`DEAP-*`, e.g. `DEAP-uas-infrastructure-safety`)**: Generate distinct README scaffolding declaring `DOMAIN_DISTRIBUTION_TEMPLATE`, enforcing the clean landing zone invariant, and providing the authoritative command for customers to clone into customer workspaces.
3. **Customer Application Workspaces (`uav-*`, e.g. `uav-011`)**: Generate distinct README scaffolding declaring `DOWNSTREAM_CUSTOMER_PROJECT`, eliminating circular self-cloning instructions, and documenting project-specific execution commands (`verify_downstream_baseline.py`, Step 0.0 ingestion, in-place update `bash scripts/install_pipeline.sh .`).

## User Authorization
> **Continuous Execution Authorized:** The user prompt explicitly provides "PROCEED", fully authorizing continuous execution through all documented work packages, subagent dispatches, verification tests, and git pushes.

---

## Proposed Changes

### Work Package 1: Upstream Spec Core Compiler README Overhaul (R1)
- **Target Files:**
  - `README.md` in `DEAP01-spec-core`
- **Action Details:**
  1. Purge broken manual snippets: Delete Section 5.4's ~80-line fragile inline Python monkeypatching script and manual `cp` loops.
  2. Compiler-centric focus: Document how to run and verify the compiler itself (`python3 scripts/compile_sysml.py`, `pytest`), and document the authoritative propagation commands for maintainers:
     - Local propagation: `bash scripts/install_pipeline.sh <path-to-domain-template>`
     - Remote bootstrap: `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler`
  3. Remove conflated domain content: Eliminate customer onboarding commands that clone domain repositories from upstream compiler quickstart sections.
  4. Ensure all code fences contain pure, valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

### Work Package 2: Dynamic Role Detection & Scaffolding in `scripts/install_pipeline.sh` (R2, R3)
- **Target Files:**
  - `scripts/install_pipeline.sh`
- **Action Details:**
  1. Detect repository tier dynamically:
     - Detect `DOMAIN_DISTRIBUTION_TEMPLATE`: e.g. target repository name starts with `DEAP-` or `--role domain-template` is passed, or target directory has `.pipeline/upstream/` absent and schema/ landing zone.
     - Detect `DOWNSTREAM_CUSTOMER_PROJECT`: e.g. target repository name starts with `uav-`, customer project naming, or default customer workspace.
  2. For Domain Distribution Templates (`DEAP-*`):
     - Declare repository classification as `DOMAIN_DISTRIBUTION_TEMPLATE`.
     - Maintain the clean landing zone invariant in docs.
     - Document authoritative customer onboarding command:
       `git clone <this-domain-template-remote-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
  3. For Customer Application Workspaces (`uav-*`):
     - Declare repository classification as `DOWNSTREAM_CUSTOMER_PROJECT`.
     - Eliminate circular instructions telling customer to clone `uav-011` into `uav-011`.
     - Document project-specific operational commands:
       - Baseline verification: `python3 scripts/verify_downstream_baseline.py`
       - Step 0.0 Level 0 OEM Ground Truth Ingestion: `python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py`
       - In-place pipeline tooling update: `bash scripts/install_pipeline.sh .`
  4. Ensure pure executable shell syntax across all generated template blocks with zero unescaped parentheses in comments.

### Work Package 3: Test Verification, Baseline Conformance, and Validation
- **Target Files:**
  - `tests/test_install_pipeline.py` or new unit tests in `tests/test_readme_scaffolding.py`
  - `scripts/verify_downstream_baseline.py`
- **Action Details:**
  1. Add unit/integration tests verifying:
     - `DEAP01-spec-core/README.md` contains zero inline multi-line Python scripts and zero domain clone commands.
     - `scripts/install_pipeline.sh` generates distinct READMEs for domain templates vs customer projects.
     - Customer project README contains zero circular clone commands and correctly documents in-place update `bash scripts/install_pipeline.sh .`.
     - Domain template README contains the proper customer bootstrap command.
     - All markdown files satisfy syntax and code fence invariants.
  2. Run `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all checks pass cleanly.
  3. Run full project test suite (`pytest`).

---

## Subagent Delegation Strategy
Per `.agents/AGENTS.md` and project rules:
1. **Survey / Exploration (`teamwork_preview_explorer`)**:
   - Map existing sections of `README.md`, `scripts/install_pipeline.sh` README generation logic, and existing tests.
2. **Worker 1 (`teamwork_preview_worker`)**:
   - Overhaul `README.md` in `DEAP01-spec-core` (WP1).
3. **Worker 2 (`teamwork_preview_worker`)**:
   - Implement dynamic tier detection and distinct README scaffolding in `scripts/install_pipeline.sh` (WP2).
4. **Worker 3 (`teamwork_preview_worker`)**:
   - Implement unit and regression tests in `tests/` and verify with `scripts/verify_downstream_baseline.py` (WP3).
5. **Reviewers & Challengers & Forensic Auditor (`teamwork_preview_reviewer`, `teamwork_preview_challenger`, `teamwork_preview_auditor`)**:
   - Multi-agent review and empirical verification gate.

---

## Verification Plan
### Automated Tests
- `pytest tests/`
- `python3 scripts/verify_downstream_baseline.py --no-domain`
- Markdown syntax and code fence checks (zero unescaped parentheses in comments, zero unquoted angle-bracket placeholders).
- Git diff verification: `git diff origin/main`.
