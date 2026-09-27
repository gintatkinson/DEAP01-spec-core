## 2026-09-27T15:38:55Z

You are the Project Orchestrator for DEAP01-spec-core.

Identity: orchestrator
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_10
Parent Sentinel: 51593246-6e8c-4cb0-8524-2d76e85cb74c
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Native Skill: skills/spec-orchestrator/SKILL.md
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Your task is to orchestrate the implementation of the user request recorded under the latest timestamp header in /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md:
"Perform a multi-agent adversarial audit and remediation of README.md and associated installer scaffolding templates in DEAP01-spec-core, resolving architecture tier numbering contradictions, heading ordering defects, and upstream vs. downstream repository execution boundaries."

## Requirements to Orchestrate:

### R1. Forensic Audit of README.md & Installer Scaffolding
- Deploy an adversarial auditor subagent to perform a comprehensive audit of `README.md` and the README scaffolding logic in `scripts/install_pipeline.sh`.
- Catalog all structural defects, including contradictory tier numbering, inverted heading hierarchies, broken/outdated test citations, and repository classification boundary ambiguities.

### R2. Architecture Tier Hierarchy & Heading Normalization
- Remediate the tier numbering contradictions in `README.md` (Sections 1.2 & 5.4) and ASCII topology diagrams where both the Upstream Compiler and Domain Distribution Templates are labeled "Tier 1":
  * Normalize to a clean, unambiguous three-tier architecture:
    - Tier 1: Upstream Specification Core Compiler (`DEAP01-spec-core`)
    - Tier 2: Domain Distribution Templates (`DEAP-*`)
    - Tier 3: Customer Application Workspaces (`uav-*`)
  * Alternatively, clearly distinguish the two propagation boundaries (Tier 1 Compiler Propagation vs. Tier 2 Customer Onboarding) from repository classifications.
- Correct the out-of-order heading sequence in Section 1 so `1.1 Primary Commercial Toolchain Integration` logically precedes `1.2 System Architecture & Repository Boundaries`.

### R3. Upstream Compiler vs. Downstream Prompt Boundary Hardening
- Clarify Section 9.4 (Pipeline 2 Operator Prompts):
  * Explicitly specify that Pipeline 2 (Autonomous Feature Implementation for Flutter/ROS2/PX4 and Digital Twin Simulation) is strictly intended for execution within downstream customer application workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`, e.g. `uav-*`).
  * Remove ambiguous statements implying that concrete application features or simulation drivers may be executed directly in `UPSTREAM_SPEC_CORE_COMPILER`.

### R4. Automated Regression & Scaffolding Test Verification
- Run `python3 -m unittest tests/test_readme_scaffolding.py` and ensure all scaffolding tests pass with exit code 0.
- Update `tests/test_readme_scaffolding.py` to assert the corrected heading hierarchy, tier definitions, and absence of contradictory tier labels.
- Run `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all baseline checks pass cleanly with exit code 0.

### R5. Git Stage, Neutral Citation Commit & Remote Synchronization
- Verify zero issue-closing verbs across commit messages via `python3 scripts/verify_commit_messages.py --head`.
- Stage all changes, commit using neutral issue citations:
  `git commit -m "docs(readme): normalize architecture tiers, fix heading sequence, and harden repository boundary (refs #371, refs #368)"`
- Push to GitHub `origin/main` and verify that `git diff origin/main` returns exactly 0 bytes.
- Have the Victory Auditor independently confirm the 0-byte remote diff and gate passes before declaring victory.

PROCEED

## Acceptance Criteria:
### Documentation & Scaffolding Integrity
- [ ] No contradictory "Tier 1" labels remain across `README.md` and `scripts/install_pipeline.sh`; tiers 1, 2, and 3 are distinctly and consistently defined.
- [ ] Section 1.1 precedes Section 1.2 in `README.md`.
- [ ] Pipeline 2 execution guidance explicitly restricts autonomous feature implementation to downstream customer workspaces.

### Verification & Automated Gates
- [ ] `python3 -m unittest tests/test_readme_scaffolding.py` passes with exit code 0.
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` passes with exit code 0.
- [ ] `python3 scripts/verify_commit_messages.py --head` verifies zero issue-closing verbs.

### Git & Remote Synchronization
- [ ] Commit message strictly matches `docs(readme): normalize architecture tiers, fix heading sequence, and harden repository boundary (refs #371, refs #368)`.
- [ ] Changes are successfully pushed to `origin/main`.
- [ ] `git diff origin/main` returns exactly 0 bytes.
