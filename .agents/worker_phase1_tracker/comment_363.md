### Verification Evidence — Resolved in 4eedb5b, 196512d, ce82ef4

The tooling defect reported in #363 where `install_pipeline.sh` synthesized non-existent GitLab URLs for GitHub domain distribution templates has been fully remediated and verified:

1. **Provider Decoupling & Parameter Support**:
   - `scripts/install_pipeline.sh` decouples the target customer workspace provider (`--provider gitlab`) from the upstream domain distribution template repository origin.
   - Added `--domain-url <URL>` and `--domain-name <NAME>` CLI arguments (supporting both whitespace and equals syntax).
   - Downstream onboarding commands now default to the canonical GitHub domain repository (`https://github.com/gintatkinson/DEAP-*.git`) even when the customer project is hosted on GitLab, preventing invalid `gitlab.com` URL synthesis.
2. **Role Precedence**:
   - Commit `ce82ef4` ensures that passing `--domain-url` or `--domain-name` correctly configures template role detection and onboarding instructions.
3. **Empirical Test Verification**:
   - `python3 -m pytest tests/test_domain_url_synthesis.py` executed cleanly: 9 passed in 8.40s (exit code 0).

Status: `Fixed / Resolved` (marking with `status:fixed-resolved` label; left open for Product Owner review per .pipeline/constitution.md:161).
