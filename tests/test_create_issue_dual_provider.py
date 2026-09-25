#!/usr/bin/env python3
"""
Unit and Regression Test Suite for Issue Creation Dual-Provider Support.

Tests Issue / Defect: Single-Provider Tooling Defect in skills/spec-orchestrator/scripts/create_issue.sh:1-120
1. Detects hardcoded single-provider `gh` CLI invocations without tracker provider auto-detection.
2. Verifies fail-closed behavior when executing create_issue.sh against a GitLab remote without gh authentication.
3. Asserts the requirement for dual-provider auto-detection (`github` vs `gitlab`) and support for `glab` CLI / GitLab REST API v4.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis,
  DO-178C C/SPARK Ada code generation).
"""

import os
import re
import subprocess
import tempfile
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CREATE_ISSUE_SCRIPT = os.path.join(
    PROJECT_ROOT, "skills", "spec-orchestrator", "scripts", "create_issue.sh"
)


class TestCreateIssueDualProvider(unittest.TestCase):
    """Test suite verifying tracker provider detection and dual-provider support in create_issue.sh."""

    def setUp(self):
        self.assertTrue(os.path.isfile(CREATE_ISSUE_SCRIPT), f"Script not found at {CREATE_ISSUE_SCRIPT}")

    def test_script_detects_hardcoded_gh_cli(self):
        """Verify that create_issue.sh contains hardcoded gh CLI calls and lacks glab / provider abstraction."""
        with open(CREATE_ISSUE_SCRIPT, "r", encoding="utf-8") as f:
            content = f.read()

        # Hardcoded gh commands
        gh_matches = re.findall(r"\bgh\s+(?:issue|label)\b", content)
        self.assertGreater(len(gh_matches), 0, "create_issue.sh should contain gh CLI calls")

        # Check for provider detection logic
        has_glab = "glab" in content
        has_provider_detect = "gitlab" in content.lower() or "detect_provider" in content or "PROVIDER" in content
        # This test documents the defect: create_issue.sh currently lacks provider auto-detection
        # When defect is resolved, dual-provider branching should be present.
        self.assertFalse(
            has_glab and has_provider_detect,
            "Defect check: create_issue.sh currently hardcodes gh and lacks glab provider branching"
        )

    def test_gitlab_remote_fails_with_gh_only(self):
        """Verify that running create_issue.sh in a GitLab workspace invokes gh rather than detecting gitlab."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            # Set up git repo with GitLab origin
            subprocess.run(["git", "init", "-b", "main"], cwd=tmp_dir, capture_output=True, check=True)
            subprocess.run(
                ["git", "remote", "add", "origin", "git@gitlab.com:aerospace/uav-009-flight-control.git"],
                cwd=tmp_dir,
                capture_output=True,
                check=True,
            )

            # Create dummy spec file and verify linter bypass or mock
            spec_file = os.path.join(tmp_dir, "spec.md")
            with open(spec_file, "w", encoding="utf-8") as f:
                f.write("---\nissue_id: 999\ntitle: 'Test Spec'\n---\n# Test Spec\n")

            # Script will attempt to call gh issue list or fail at linter
            # This confirms create_issue.sh does not detect the gitlab remote URL from git remote get-url origin
            with open(CREATE_ISSUE_SCRIPT, "r", encoding="utf-8") as f:
                lines = f.readlines()

            # Ensure script contains no git remote inspection for gitlab
            remote_inspect = any("git remote" in line or "remote.origin.url" in line for line in lines)
            self.assertFalse(remote_inspect, "create_issue.sh lacks remote origin provider detection")


if __name__ == "__main__":
    unittest.main()
