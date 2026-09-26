# Dispatch for Cluster B Implementer (R3: Dual-Provider Tooling & Installer Hardening)

## Objective
Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_b
Original Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Triage Report: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_phase1/triage_report.md

Exclusive Write Ownership:
- `tests/test_polyrepo_propagation_gate.py`

Tasks:
1. Remediate #372:
   - Implement `tests/test_polyrepo_propagation_gate.py` to establish an automated cross-repository integration test and polyrepo rollout gate.
   - The test must assert that `scripts/install_pipeline.sh`:
     a. Correctly provisions a downstream GitLab sandbox repository with working dual-provider issue creation (`create_issue.sh`).
     b. Accurately passes `--provider gitlab` with `--domain-url` preserving GitHub upstream domain repositories and avoiding invalid GitLab URL synthesis.
     c. Enforces downstream baseline verification readiness in GitLab mode.
2. TDD Verification:
   - Run `python3 -m pytest tests/test_polyrepo_propagation_gate.py` and ensure 100% pass rate.
   - Confirm zero regressions in `tests/test_create_issue_dual_provider.py` and `tests/test_domain_url_synthesis.py`.
3. Provide handoff report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_cluster_b/handoff.md`.

PROCEED

## 2026-09-26T20:09:07Z
Context: Cluster B Remediation (#372: Polyrepo Propagation Gate)
Content: Checking on your status. Have you finished debugging tests/test_polyrepo_propagation_gate.py and generating your handoff report?
Action: Please report current status and write handoff.md if complete.

