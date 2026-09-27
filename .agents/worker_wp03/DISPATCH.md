## 2026-09-27T16:00:00Z

# Task Assignment: WP-03 Automated Regression & Scaffolding Test Verification

You are the Test Verification Worker for WP-03.
Target File Owned:
- `/Users/perkunas/jail/DEAP01-spec-core/tests/test_readme_scaffolding.py`

Tasks:
1. Update `tests/test_readme_scaffolding.py` per the specifications in `.agents/auditor_wp01/handoff.md` Section 5.2 and `implementation_plan.md` WP-03:
   - Update docstrings at lines 381 and 482 to reference Tier 2 Domain Distribution Templates and Tier 3 Customer Application Workspaces.
   - In `TestUpstreamCompilerReadme`, add:
     * `test_upstream_readme_section_1_heading_order_and_hierarchy`
     * `test_upstream_readme_three_tier_architecture_normalization`
     * `test_upstream_readme_pipeline_2_prompt_boundary_confinement`
   - In `TestDomainDistributionTemplateScaffolding.test_domain_template_scaffolding_explicit_role`:
     * Assert `content` contains `As a **Tier 2 Domain Distribution Template**`.
     * Assert `content` does not contain `Tier 1 Domain Distribution Template`.
   - In `TestCustomerWorkspaceScaffolding.test_customer_workspace_scaffolding_explicit_role`:
     * Assert `content` contains `As a **Tier 3 Customer Application Workspace**`.
     * Assert `content` does not contain `Tier 2 Customer Application Workspace`.
2. Run automated test suites:
   - `python3 -m unittest tests/test_readme_scaffolding.py` (assert exit code 0).
   - `python3 scripts/verify_downstream_baseline.py --no-domain` (assert exit code 0).
   - `python3 -m pytest tests/` (assert 0 failures).
3. Deliver complete report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp03/handoff.md`.
4. Notify orchestrator when done via send_message.

PROCEED
