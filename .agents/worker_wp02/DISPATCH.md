# Task Assignment: WP-02 Architecture Tier Hierarchy, Heading Normalization & Repository Boundary Hardening

You are the Documentation & Tooling Worker for WP-02.
Target Files:
- `/Users/perkunas/jail/DEAP01-spec-core/README.md`
- `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`

Perform the exact remediation specified in:
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_wp01/handoff.md` Section 5.1
- `/Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md` WP-02

Summary of Actions:
1. README.md:
   - Normalize Section 1: Reposition Section 1.1 `### 1.1 Primary Commercial Toolchain Integration` before Section 1.2.
   - Update Section 1.2 to `### 1.2 Three-Tier Architecture & Repository Boundaries: Upstream Compiler vs. Domain Templates vs. Customer Workspaces`.
   - Update ASCII diagram and text in Section 1.2 to clean three-tier architecture:
     * Tier 1: Upstream Specification Core Compiler (`DEAP01-spec-core`, `UPSTREAM_SPEC_CORE_COMPILER`)
     * Tier 2: Domain Distribution Templates (`DEAP-*`, `DOMAIN_DISTRIBUTION_TEMPLATE`)
     * Tier 3: Customer Application Workspaces (`uav-*`, `DOWNSTREAM_CUSTOMER_PROJECT`)
   - Update Section 4: line 152 to `#### Tier 2: Domain Distribution Template Repository`, line 175 to `Tier 3 Customer Application Workspaces`, and line 177 to `### 4.1 Supported Tier 2 Domain Distribution Templates`.
   - Update Section 5.4: update heading, text, ASCII diagram, and anchor link on line 291 to `#41-supported-tier-2-domain-distribution-templates-canonical-taxonomies`.
   - Update Section 9.4: Add Execution Boundary Invariant admonition restricting Pipeline 2 to downstream customer workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`). Remove `(or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` from lines 1094, 1149, 1180.
2. scripts/install_pipeline.sh:
   - Line 882: Replace `As a **Tier 1 Domain Distribution Template**` with `As a **Tier 2 Domain Distribution Template**`.
   - Line 953: Replace `As a **Tier 2 Customer Application Workspace**` with `As a **Tier 3 Customer Application Workspace**`.
   - Lines 1430, 1484, 1516: Replace `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` with `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`.

Deliver completion handoff to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp02/handoff.md`.
Notify orchestrator when done via send_message.

PROCEED

## 2026-09-27T15:53:07Z
Received dispatch for Work Package WP-02: Architecture Tier Hierarchy, Heading Normalization & Repository Boundary Hardening.
Target files: README.md, scripts/install_pipeline.sh.
Following exact remediation steps from .agents/auditor_wp01/handoff.md Section 5.1.
