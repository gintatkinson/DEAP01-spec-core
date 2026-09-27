# Task Assignment: WP-01 Forensic Adversarial Audit

You are the Adversarial Forensic Auditor for WP-01.
Perform a comprehensive forensic audit of README.md, scripts/install_pipeline.sh, and tests/test_readme_scaffolding.py.
Catalog contradictory tier numbering, inverted heading hierarchies, broken/outdated test citations, and repository boundary ambiguities.
Deliver your report to /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp01/handoff.md.

## 2026-09-27T15:43:09Z

Execute view_file on skills/adversarial-code-auditor/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp01
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Original Request Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan Path: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md

You are assigned Work Package WP-01: Forensic Audit of README.md & Installer Scaffolding.
Perform an adversarial forensic audit of `README.md`, `scripts/install_pipeline.sh`, and `tests/test_readme_scaffolding.py` in DEAP01-spec-core.

Audit Scope:
1. Contradictory Architecture Tier Numbering:
   - Identify every location in `README.md` (Sections 1.2, 5.4, ASCII diagrams) and `scripts/install_pipeline.sh` (lines 882, 953) where both Upstream Compiler and Domain Distribution Templates are labeled "Tier 1", and Customer Application Workspaces are labeled "Tier 2".
   - Specify the exact text changes needed to normalize to a clean three-tier architecture:
     * Tier 1: Upstream Specification Core Compiler (`DEAP01-spec-core`)
     * Tier 2: Domain Distribution Templates (`DEAP-*`)
     * Tier 3: Customer Application Workspaces (`uav-*`)
2. Inverted Heading Sequence:
   - Identify the heading order defect in Section 1 of `README.md` where `1.1 Primary Commercial Toolchain Integration` appears after `1.2 Two-Tier Architecture Boundary`.
   - Specify the exact repositioning needed so 1.1 logically precedes 1.2.
3. Upstream Compiler vs. Downstream Prompt Boundary Hardening:
   - Identify the repository classification boundary violations in Section 9.4 (Pipeline 2 Operator Prompts) of `README.md` (lines 1094, 1149, 1180) where prompt preambles state `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)`.
   - Specify the exact removals and clarifications to ensure Pipeline 2 (Autonomous Feature Implementation for Flutter/ROS2/PX4 and Digital Twin Simulation) is strictly confined to downstream customer application workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`) and never executed in `UPSTREAM_SPEC_CORE_COMPILER`.
4. Test Suite Audit:
   - Inspect `tests/test_readme_scaffolding.py` for assertions or docstrings reflecting legacy two-tier models or missing assertions for 1.1 preceding 1.2 and absence of contradictory tier labels.
5. If new defects require tracker issues, submit via `gh issue create` and `glab issue create` adhering to the non-closure commit and issue naming rules.

Deliver your complete findings and verified remediation specifications in `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp01/handoff.md`.
Notify orchestrator when complete via send_message.

PROCEED
