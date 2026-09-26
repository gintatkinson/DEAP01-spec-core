### Verification Evidence — Resolved in 14932ff, 080fc49, 90fe8fa

The defect reported in #368 regarding downstream onboarding rule-shortcutting and lack of a consolidated governance manifest has been fully remediated and verified:

1. **Automated Bundle Compilation**:
   - `scripts/install_pipeline.sh` (lines 483-530) compiles all active rules from `rules/*.md` into `.pipeline/ACTIVE_RULES_BUNDLE.md` at installation time.
   - The compiled manifest includes a Table of Contents, standardized markdown anchors, and unified rule sections.
2. **README Scaffolding & Governance Mandates**:
   - Downstream README templates in `scripts/install_pipeline.sh` (lines 894, 930, 965, 1011, 1461, 1495, 1524) explicitly instruct agents to execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md` in a single read.
   - Upgrade detection logic (lines 697, 707) enforces regeneration when `ACTIVE_RULES_BUNDLE.md` is absent.
3. **Repository Manifest**:
   - `.pipeline/ACTIVE_RULES_BUNDLE.md` is committed to the repository (1849 lines).
4. **Empirical Test Verification**:
   - `python3 -m pytest tests/test_readme_scaffolding.py` executed cleanly: 31 passed in 31.67s (exit code 0).

Status: `Fixed / Resolved` (marking with `status:fixed-resolved` label; left open for Product Owner review per .pipeline/constitution.md:161).
