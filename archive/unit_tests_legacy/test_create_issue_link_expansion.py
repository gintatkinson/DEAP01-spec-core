"""Unit tests for skills/spec-orchestrator/scripts/create_issue.sh relative link expansion.

Issue #244: Relative markdown links unexpanded in issue body causing 404 on tracker.
Verifies that create_issue.sh expands relative markdown links (e.g. ../epics/..., ../../schema/...)
to full web blob URLs (GitHub /blob/main/... or GitLab /-/blob/main/...) before creating issues.
"""

import json
import os
import shutil
import stat
import subprocess
import tempfile
import unittest

SCRIPT_PATH = os.path.abspath(
    os.path.join(
        os.path.dirname(__file__),
        "..",
        "skills",
        "spec-orchestrator",
        "scripts",
        "create_issue.sh",
    )
)


class TestCreateIssueLinkExpansion(unittest.TestCase):
    """Test suite verifying that create_issue.sh pre-expands relative markdown links to full blob URLs."""

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.workspace_dir = self.temp_dir.name

        # Create docs structure
        self.docs_dir = os.path.join(self.workspace_dir, "docs")
        self.epics_dir = os.path.join(self.docs_dir, "epics")
        self.features_dir = os.path.join(self.docs_dir, "features")
        self.schema_dir = os.path.join(self.workspace_dir, "schema")
        self.scripts_dir = os.path.join(self.workspace_dir, "scripts")
        self.bin_dir = os.path.join(self.workspace_dir, "bin")

        os.makedirs(self.epics_dir, exist_ok=True)
        os.makedirs(self.features_dir, exist_ok=True)
        os.makedirs(self.schema_dir, exist_ok=True)
        os.makedirs(self.scripts_dir, exist_ok=True)
        os.makedirs(self.bin_dir, exist_ok=True)

        # Initialize git repo with main branch
        subprocess.run(["git", "init", "-b", "main"], cwd=self.workspace_dir, capture_output=True, check=True)


        # Create target referenced files
        self.epic_file = os.path.join(self.epics_dir, "epic-01-core.md")
        with open(self.epic_file, "w", encoding="utf-8") as f:
            f.write("# Epic 01 Core\n")

        self.schema_file = os.path.join(self.schema_dir, "platform.sysml")
        with open(self.schema_file, "w", encoding="utf-8") as f:
            f.write("// SysML Model\n")

        # Create spec file containing relative markdown links
        self.feature_file = os.path.join(self.features_dir, "feat-01-auth.md")
        self.feature_content = (
            "---\n"
            "issue_id: 101\n"
            "title: 'Feature 01: User Authentication'\n"
            "type: feature\n"
            "---\n\n"
            "# Feature 01: User Authentication\n\n"
            "## 1. Description\n"
            "User authentication service.\n\n"
            "## 2. Acceptance Criteria\n"
            "- Scenario 1: Valid Login\n\n"
            "## 3. Source References\n"
            "- Realizes: [Epic 01 Core](../epics/epic-01-core.md)\n"
            "- Schema: [Platform Schema](../../schema/platform.sysml)\n"
        )
        with open(self.feature_file, "w", encoding="utf-8") as f:
            f.write(self.feature_content)

        # Captured issue payload file for mock gh
        self.captured_body_file = os.path.join(self.workspace_dir, "captured_body.md")
        self.captured_args_file = os.path.join(self.workspace_dir, "captured_args.json")

        # Create mock gh script
        self.mock_gh = os.path.join(self.bin_dir, "gh")
        mock_gh_script = f"""#!/bin/bash
set -e

# Mock gh CLI
if [ "$1" = "issue" ] && [ "$2" = "list" ]; then
    exit 0
fi

if [ "$1" = "label" ] && [ "$2" = "list" ]; then
    printf "feature\\t#0366d6\\n"
    exit 0
fi

if [ "$1" = "label" ] && [ "$2" = "create" ]; then
    exit 0
fi

if [ "$1" = "issue" ] && [ "$2" = "create" ]; then
    # Record arguments
    python3 -c '
import sys, os, json
args = sys.argv[1:]
body_file = None
for i, a in enumerate(args):
    if a == "--body-file" and i + 1 < len(args):
        body_file = args[i + 1]
with open(r"{self.captured_args_file}", "w", encoding="utf-8") as f:
    json.dump({{"args": args, "body_file": body_file}}, f)
if body_file and os.path.exists(body_file):
    with open(body_file, "r", encoding="utf-8") as src, open(r"{self.captured_body_file}", "w", encoding="utf-8") as dst:
        dst.write(src.read())
' "$@"
    echo "https://github.com/gintatkinson/DEAP01-spec-core/issues/101"
    exit 0
fi

exit 0
"""
        with open(self.mock_gh, "w", encoding="utf-8") as f:
            f.write(mock_gh_script)
        os.chmod(self.mock_gh, os.stat(self.mock_gh).st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

    def tearDown(self):
        self.temp_dir.cleanup()

    def _run_create_issue(self, file_path, label, title, repo=None, extra_env=None):
        env = os.environ.copy()
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"
        if extra_env:
            env.update(extra_env)

        cmd = ["bash", SCRIPT_PATH, file_path, label, title]
        if repo:
            cmd.append(repo)

        return subprocess.run(
            cmd,
            cwd=self.workspace_dir,
            env=env,
            capture_output=True,
            text=True,
        )

    def test_create_issue_expands_relative_links_github(self):
        """Verifies create_issue.sh expands relative links to GitHub blob URLs before issue creation."""
        rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "github",
            }
        }
        with open(os.path.join(self.workspace_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
            json.dump(rules, f)

        res = self._run_create_issue(
            self.feature_file,
            "feature",
            "Feature 01: User Authentication",
            repo="gintatkinson/DEAP01-spec-core",
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.exists(self.captured_body_file), "Captured body file was not created by mock gh")
        with open(self.captured_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        # The issue body must have expanded relative links to full github blob URLs
        self.assertIn(
            "[Epic 01 Core](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/epics/epic-01-core.md)",
            body,
            "Relative link to epic was not expanded to GitHub blob URL in issue payload"
        )
        self.assertIn(
            "[Platform Schema](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/schema/platform.sysml)",
            body,
            "Relative link to schema was not expanded to GitHub blob URL in issue payload"
        )
        self.assertNotIn(
            "../epics/epic-01-core.md",
            body,
            "Raw unexpanded relative link to epic remained in issue payload"
        )
        self.assertNotIn(
            "../../schema/platform.sysml",
            body,
            "Raw unexpanded relative link to schema remained in issue payload"
        )

    def test_create_issue_expands_relative_links_gitlab(self):
        """Verifies create_issue.sh expands relative links to GitLab blob URLs when configured."""
        rules = {
            "meta": {"upstream_repository": "gintatkinson/uav-008"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.com",
            }
        }
        with open(os.path.join(self.workspace_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
            json.dump(rules, f)

        res = self._run_create_issue(
            self.feature_file,
            "feature",
            "Feature 01: User Authentication",
            repo="gintatkinson/uav-008",
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.exists(self.captured_body_file), "Captured body file was not created by mock gh")
        with open(self.captured_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        # GitLab blob URLs contain /-/blob/
        self.assertIn(
            "[Epic 01 Core](https://gitlab.com/gintatkinson/uav-008/-/blob/main/docs/epics/epic-01-core.md)",
            body,
            "Relative link to epic was not expanded to GitLab blob URL in issue payload"
        )
        self.assertIn(
            "[Platform Schema](https://gitlab.com/gintatkinson/uav-008/-/blob/main/schema/platform.sysml)",
            body,
            "Relative link to schema was not expanded to GitLab blob URL in issue payload"
        )

    def test_create_issue_preserves_external_urls_and_anchors(self):
        """Verifies create_issue.sh preserves external web URLs and expands links with anchors."""
        mixed_spec_file = os.path.join(self.features_dir, "feat-02-mixed.md")
        mixed_content = (
            "---\n"
            "issue_id: 102\n"
            "title: 'Feature 02: Mixed Links'\n"
            "type: feature\n"
            "---\n\n"
            "# Feature 02: Mixed Links\n\n"
            "## References\n"
            "- [RFC 7519](https://tools.ietf.org/html/rfc7519)\n"
            "- [Section 2](../epics/epic-01-core.md#section-2)\n"
            "- [Internal Anchor](#local-anchor)\n"
        )
        with open(mixed_spec_file, "w", encoding="utf-8") as f:
            f.write(mixed_content)

        rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {"provider": "github"}
        }
        with open(os.path.join(self.workspace_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
            json.dump(rules, f)

        res = self._run_create_issue(
            mixed_spec_file,
            "feature",
            "Feature 02: Mixed Links",
            repo="gintatkinson/DEAP01-spec-core",
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        with open(self.captured_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        self.assertIn("[RFC 7519](https://tools.ietf.org/html/rfc7519)", body)
        self.assertIn(
            "[Section 2](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/epics/epic-01-core.md#section-2)",
            body
        )
        self.assertIn("[Internal Anchor](#local-anchor)", body)

    def test_create_issue_custom_repo_override(self):
        """Verifies that passing a custom repo parameter to create_issue.sh overrides codebase_rules."""
        rules = {
            "meta": {"upstream_repository": "default-org/default-repo"},
            "tracker_rules": {"provider": "github"}
        }
        with open(os.path.join(self.workspace_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
            json.dump(rules, f)

        res = self._run_create_issue(
            self.feature_file,
            "feature",
            "Feature 01: User Authentication",
            repo="custom-org/custom-repo",
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        with open(self.captured_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        self.assertIn(
            "[Epic 01 Core](https://github.com/custom-org/custom-repo/blob/main/docs/epics/epic-01-core.md)",
            body
        )

    def test_temporary_body_file_is_cleaned_up(self):
        """Verifies that the temporary file created for body expansion is cleaned up after create_issue.sh exits."""
        res = self._run_create_issue(
            self.feature_file,
            "feature",
            "Feature 01: User Authentication",
            repo="gintatkinson/DEAP01-spec-core",
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.exists(self.captured_args_file))
        with open(self.captured_args_file, "r", encoding="utf-8") as f:
            data = json.load(f)

        passed_body_file = data.get("body_file")
        self.assertIsNotNone(passed_body_file)
        self.assertNotEqual(
            passed_body_file,
            self.feature_file,
            "create_issue.sh must pass a preprocessed temporary body file, not the raw local file"
        )
        # Verify the temporary file was cleaned up on exit
        self.assertFalse(
            os.path.exists(passed_body_file),
            f"Temporary body file {passed_body_file} was not cleaned up on script exit"
        )


if __name__ == "__main__":
    unittest.main()


