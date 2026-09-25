#!/usr/bin/env python3
"""
Unit and Regression Test Suite for Issue Creation Dual-Provider Support.

Comprehensively verifies dual-provider (`gh` and `glab`) support in
`skills/spec-orchestrator/scripts/create_issue.sh`:
1. Remote origin URL auto-detection (`gitlab.com` -> `gitlab`, `github.com` -> `github`).
2. CLI flag `--provider <github|gitlab>` override and `TRACKER_PROVIDER` env override.
3. Precedence hierarchy (CLI flag > env var > remote URL > default).
4. CI environment detection (`GITLAB_CI` -> `gitlab`).
5. Mock CLI command verification for both `gh` and `glab`.
6. Idempotency handling across both providers.
7. Defect analysis attachment via `--reopen-with-analysis` across both providers.
8. Provider-aware relative markdown link expansion (`/blob/` vs `/-/blob/`).
9. Label check and creation across both providers.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis,
  DO-178C C/SPARK Ada code generation).
"""

import json
import os
import stat
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
        self.temp_dir = tempfile.TemporaryDirectory()
        self.workspace_dir = self.temp_dir.name

        # Create directories
        self.docs_dir = os.path.join(self.workspace_dir, "docs")
        self.epics_dir = os.path.join(self.docs_dir, "epics")
        self.features_dir = os.path.join(self.docs_dir, "features")
        self.schema_dir = os.path.join(self.workspace_dir, "schema")
        self.bin_dir = os.path.join(self.workspace_dir, "bin")

        os.makedirs(self.epics_dir, exist_ok=True)
        os.makedirs(self.features_dir, exist_ok=True)
        os.makedirs(self.schema_dir, exist_ok=True)
        os.makedirs(self.bin_dir, exist_ok=True)

        # Create target referenced files
        self.epic_file = os.path.join(self.epics_dir, "epic-01-core.md")
        with open(self.epic_file, "w", encoding="utf-8") as f:
            f.write("# Epic 01 Core\n")

        self.schema_file = os.path.join(self.schema_dir, "platform.sysml")
        with open(self.schema_file, "w", encoding="utf-8") as f:
            f.write("// SysML Model\n")

        # Create spec file
        self.feature_file = os.path.join(self.features_dir, "feat-01.md")
        self.feature_title = "Feature 01: Flight Guidance System"
        self.feature_content = (
            "---\n"
            "issue_id: 101\n"
            f"title: '{self.feature_title}'\n"
            "type: feature\n"
            "---\n\n"
            f"# {self.feature_title}\n\n"
            "## 1. Description\n"
            "Flight guidance system synthesized via MATLAB / Simulink / Stateflow / Embedded Coder.\n\n"
            "## 2. Acceptance Criteria\n"
            "- Scenario 1: Deterministic discrete step solver\n\n"
            "## 3. Source References\n"
            "- Realizes: [Epic 01 Core](../epics/epic-01-core.md)\n"
            "- Schema: [Platform Schema](../../schema/platform.sysml)\n"
        )
        with open(self.feature_file, "w", encoding="utf-8") as f:
            f.write(self.feature_content)

        # Log and capture paths
        self.gh_log_file = os.path.join(self.workspace_dir, "gh_calls.json")
        self.glab_log_file = os.path.join(self.workspace_dir, "glab_calls.json")
        self.gh_body_file = os.path.join(self.workspace_dir, "gh_body.md")
        self.glab_body_file = os.path.join(self.workspace_dir, "glab_body.md")

        # Install mock binaries
        self._install_mock_gh()
        self._install_mock_glab()

    def tearDown(self):
        self.temp_dir.cleanup()

    def _install_mock_gh(self):
        mock_gh_path = os.path.join(self.bin_dir, "gh")
        content = f"""#!/usr/bin/env python3
import sys, json, os

log_file = r"{self.gh_log_file}"
calls = []
if os.path.exists(log_file):
    try:
        with open(log_file, "r") as f:
            calls = json.load(f)
    except Exception:
        calls = []
calls.append(sys.argv[1:])
with open(log_file, "w") as f:
    json.dump(calls, f)

args = sys.argv[1:]
if len(args) >= 2 and args[0] == "issue" and args[1] == "list":
    existing = os.environ.get("MOCK_GH_EXISTING")
    if existing:
        print(existing)
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "list":
    if os.environ.get("MOCK_GH_MISSING_LABEL"):
        sys.exit(0)
    print("feature\\t0366d6\\tfeature specification")
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "create":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "create":
    body_capture = r"{self.gh_body_file}"
    for i, a in enumerate(args):
        if a == "--body-file" and i + 1 < len(args):
            if os.path.exists(args[i+1]):
                with open(args[i+1], "r", encoding="utf-8") as src, open(body_capture, "w", encoding="utf-8") as dst:
                    dst.write(src.read())
    print("https://github.com/aerospace/uav-009-flight-control/issues/101")
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "reopen":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "comment":
    sys.exit(0)

sys.exit(0)
"""
        with open(mock_gh_path, "w", encoding="utf-8") as f:
            f.write(content)
        os.chmod(mock_gh_path, os.stat(mock_gh_path).st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

    def _install_mock_glab(self):
        mock_glab_path = os.path.join(self.bin_dir, "glab")
        content = f"""#!/usr/bin/env python3
import sys, json, os

log_file = r"{self.glab_log_file}"
calls = []
if os.path.exists(log_file):
    try:
        with open(log_file, "r") as f:
            calls = json.load(f)
    except Exception:
        calls = []
calls.append(sys.argv[1:])
with open(log_file, "w") as f:
    json.dump(calls, f)

args = sys.argv[1:]
if len(args) >= 2 and args[0] == "issue" and args[1] == "list":
    if "--output" in args and "json" in args:
        print("[]")
        sys.exit(0)
    existing = os.environ.get("MOCK_GLAB_EXISTING")
    if existing:
        print(existing)
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "list":
    if os.environ.get("MOCK_GLAB_MISSING_LABEL"):
        sys.exit(0)
    print("feature\\t#0366d6\\tfeature specification")
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "create":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "create":
    body_capture = r"{self.glab_body_file}"
    for i, a in enumerate(args):
        if a == "--description" and i + 1 < len(args):
            with open(body_capture, "w", encoding="utf-8") as dst:
                dst.write(args[i+1])
    print("https://gitlab.com/aerospace/uav-009-flight-control/-/issues/101")
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "reopen":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "note":
    sys.exit(0)

sys.exit(0)
"""
        with open(mock_glab_path, "w", encoding="utf-8") as f:
            f.write(content)
        os.chmod(mock_glab_path, os.stat(mock_glab_path).st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

    def _init_git_repo(self, remote_url: str):
        subprocess.run(["git", "init", "-b", "main"], cwd=self.workspace_dir, capture_output=True, check=True)
        subprocess.run(["git", "remote", "add", "origin", remote_url], cwd=self.workspace_dir, capture_output=True, check=True)

    def _run_create_issue(self, extra_args=None, extra_env=None):
        env = os.environ.copy()
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"
        # Clear external tracking env vars to ensure clean test environment
        env.pop("TRACKER_PROVIDER", None)
        env.pop("PROVIDER", None)
        env.pop("GITLAB_CI", None)

        if extra_env:
            env.update(extra_env)

        cmd = ["bash", CREATE_ISSUE_SCRIPT]
        if extra_args:
            cmd.extend(extra_args)

        return subprocess.run(
            cmd,
            cwd=self.workspace_dir,
            env=env,
            capture_output=True,
            text=True,
        )

    def _get_gh_calls(self):
        if not os.path.exists(self.gh_log_file):
            return []
        with open(self.gh_log_file, "r", encoding="utf-8") as f:
            return json.load(f)

    def _get_glab_calls(self):
        if not os.path.exists(self.glab_log_file):
            return []
        with open(self.glab_log_file, "r", encoding="utf-8") as f:
            return json.load(f)

    # -------------------------------------------------------------------------
    # 1. Remote URL Auto-Detection Tests
    # -------------------------------------------------------------------------

    def test_remote_auto_detection_gitlab(self):
        """Verifies gitlab remote URL triggers gitlab provider and executes glab CLI."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([self.feature_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        glab_calls = self._get_glab_calls()
        gh_calls = self._get_gh_calls()

        self.assertGreater(len(glab_calls), 0, "glab CLI was not called in GitLab workspace")
        self.assertEqual(len(gh_calls), 0, "gh CLI was unexpectedly called in GitLab workspace")

        # Verify glab issue list and glab issue create were executed
        subcommands = [c[0] for c in glab_calls]
        self.assertIn("issue", subcommands)
        create_calls = [c for c in glab_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 1, "glab issue create should have been called once")

    def test_remote_auto_detection_github(self):
        """Verifies github remote URL triggers github provider and executes gh CLI."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([self.feature_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        gh_calls = self._get_gh_calls()
        glab_calls = self._get_glab_calls()

        self.assertGreater(len(gh_calls), 0, "gh CLI was not called in GitHub workspace")
        self.assertEqual(len(glab_calls), 0, "glab CLI was unexpectedly called in GitHub workspace")

        create_calls = [c for c in gh_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 1, "gh issue create should have been called once")

    # -------------------------------------------------------------------------
    # 2. CLI Flag & Environment Variable Overrides
    # -------------------------------------------------------------------------

    def test_cli_flag_provider_override_gitlab(self):
        """Verifies --provider gitlab overrides a github remote URL."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([
            "--provider", "gitlab",
            self.feature_file,
            "feature",
            self.feature_title,
        ])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_glab_calls()), 0, "glab CLI should be called with --provider gitlab")
        self.assertEqual(len(self._get_gh_calls()), 0, "gh CLI should not be called with --provider gitlab")

    def test_cli_flag_provider_override_github(self):
        """Verifies --provider github overrides a gitlab remote URL."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([
            "--provider", "github",
            self.feature_file,
            "feature",
            self.feature_title,
        ])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_gh_calls()), 0, "gh CLI should be called with --provider github")
        self.assertEqual(len(self._get_glab_calls()), 0, "glab CLI should not be called with --provider github")

    def test_env_var_tracker_provider_override_gitlab(self):
        """Verifies TRACKER_PROVIDER=gitlab overrides a github remote URL."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"TRACKER_PROVIDER": "gitlab"},
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_glab_calls()), 0, "glab CLI should be called when TRACKER_PROVIDER=gitlab")
        self.assertEqual(len(self._get_gh_calls()), 0, "gh CLI should not be called when TRACKER_PROVIDER=gitlab")

    def test_env_var_tracker_provider_override_github(self):
        """Verifies TRACKER_PROVIDER=github overrides a gitlab remote URL."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"TRACKER_PROVIDER": "github"},
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_gh_calls()), 0, "gh CLI should be called when TRACKER_PROVIDER=github")
        self.assertEqual(len(self._get_glab_calls()), 0, "glab CLI should not be called when TRACKER_PROVIDER=github")

    def test_cli_flag_precedence_over_env_var(self):
        """Verifies --provider CLI flag takes precedence over TRACKER_PROVIDER env var."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            ["--provider", "gitlab", self.feature_file, "feature", self.feature_title],
            extra_env={"TRACKER_PROVIDER": "github"},
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_glab_calls()), 0, "glab CLI should be called when --provider gitlab is given")
        self.assertEqual(len(self._get_gh_calls()), 0, "gh CLI should not be called when --provider gitlab is given")

    def test_gitlab_ci_env_detection(self):
        """Verifies GITLAB_CI=1 activates gitlab provider when no other override is present."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"GITLAB_CI": "1"},
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertGreater(len(self._get_glab_calls()), 0, "glab CLI should be called when GITLAB_CI is set")
        self.assertEqual(len(self._get_gh_calls()), 0, "gh CLI should not be called when GITLAB_CI is set")

    # -------------------------------------------------------------------------
    # 3. Idempotency Handling Across Providers
    # -------------------------------------------------------------------------

    def test_gitlab_idempotency_avoids_duplicate(self):
        """Verifies existing issue with identical title in GitLab results in [IDEMPOTENT] exit 0 and no create call."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"MOCK_GLAB_EXISTING": f"101\t{self.feature_title}\tfeature\topen"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}")
        self.assertIn("[IDEMPOTENT]", res.stdout)
        self.assertIn(f"Issue #101 already carries the title '{self.feature_title}'", res.stdout)

        glab_calls = self._get_glab_calls()
        create_calls = [c for c in glab_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 0, "glab issue create must NOT be called when issue already exists")

    def test_github_idempotency_avoids_duplicate(self):
        """Verifies existing issue with identical title in GitHub results in [IDEMPOTENT] exit 0 and no create call."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"MOCK_GH_EXISTING": f"101\t{self.feature_title}\tfeature\topen"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}")
        self.assertIn("[IDEMPOTENT]", res.stdout)
        self.assertIn(f"Issue #101 already carries the title '{self.feature_title}'", res.stdout)

        gh_calls = self._get_gh_calls()
        create_calls = [c for c in gh_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 0, "gh issue create must NOT be called when issue already exists")

    # -------------------------------------------------------------------------
    # 4. Reopen with Defect Analysis Across Providers
    # -------------------------------------------------------------------------

    def test_gitlab_reopen_with_analysis(self):
        """Verifies --reopen-with-analysis reopens existing GitLab issue and posts note comment."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        dossier_path = os.path.join(self.workspace_dir, "dossier.md")
        analysis_text = "Dossier: Simulink discrete solver step singularity observed at dt=0.004s."
        with open(dossier_path, "w", encoding="utf-8") as f:
            f.write(analysis_text)

        res = self._run_create_issue(
            [
                "--reopen-with-analysis", dossier_path,
                self.feature_file,
                "feature",
                self.feature_title,
            ],
            extra_env={"MOCK_GLAB_EXISTING": f"101\t{self.feature_title}\tfeature\tclosed"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")
        self.assertIn("[REOPEN]", res.stdout)

        glab_calls = self._get_glab_calls()
        reopen_calls = [c for c in glab_calls if c[:2] == ["issue", "reopen"]]
        note_calls = [c for c in glab_calls if c[:2] == ["issue", "note"]]
        create_calls = [c for c in glab_calls if c[:2] == ["issue", "create"]]

        self.assertEqual(len(reopen_calls), 1, "glab issue reopen must be called")
        self.assertIn("101", reopen_calls[0], "glab issue reopen must target issue 101")

        self.assertEqual(len(note_calls), 1, "glab issue note must be called")
        self.assertIn("101", note_calls[0], "glab issue note must target issue 101")
        # Check message content
        msg_idx = note_calls[0].index("--message")
        self.assertIn(analysis_text, note_calls[0][msg_idx + 1])

        self.assertEqual(len(create_calls), 0, "glab issue create must NOT be called on reopen")

    def test_github_reopen_with_analysis(self):
        """Verifies --reopen-with-analysis reopens existing GitHub issue and posts comment."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        dossier_path = os.path.join(self.workspace_dir, "dossier.md")
        analysis_text = "Dossier: Simulink discrete solver step singularity observed at dt=0.004s."
        with open(dossier_path, "w", encoding="utf-8") as f:
            f.write(analysis_text)

        res = self._run_create_issue(
            [
                "--reopen-with-analysis", dossier_path,
                self.feature_file,
                "feature",
                self.feature_title,
            ],
            extra_env={"MOCK_GH_EXISTING": f"101\t{self.feature_title}\tfeature\tclosed"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")
        self.assertIn("[REOPEN]", res.stdout)

        gh_calls = self._get_gh_calls()
        reopen_calls = [c for c in gh_calls if c[:2] == ["issue", "reopen"]]
        comment_calls = [c for c in gh_calls if c[:2] == ["issue", "comment"]]
        create_calls = [c for c in gh_calls if c[:2] == ["issue", "create"]]

        self.assertEqual(len(reopen_calls), 1, "gh issue reopen must be called")
        self.assertIn("101", reopen_calls[0], "gh issue reopen must target issue 101")

        self.assertEqual(len(comment_calls), 1, "gh issue comment must be called")
        self.assertIn("101", comment_calls[0], "gh issue comment must target issue 101")
        body_idx = comment_calls[0].index("--body")
        self.assertIn(analysis_text, comment_calls[0][body_idx + 1])

        self.assertEqual(len(create_calls), 0, "gh issue create must NOT be called on reopen")

    # -------------------------------------------------------------------------
    # 5. Relative Link Expansion Across Providers
    # -------------------------------------------------------------------------

    def test_gitlab_relative_link_expansion(self):
        """Verifies GitLab issue creation receives expanded /-/blob/ URLs in issue description."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([self.feature_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.exists(self.glab_body_file), "glab description capture file should exist")
        with open(self.glab_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        self.assertIn(
            "[Epic 01 Core](https://gitlab.com/aerospace/uav-009-flight-control/-/blob/main/docs/epics/epic-01-core.md)",
            body,
            "GitLab blob URL must contain /-/blob/ and point to gitlab.com",
        )
        self.assertIn(
            "[Platform Schema](https://gitlab.com/aerospace/uav-009-flight-control/-/blob/main/schema/platform.sysml)",
            body,
            "GitLab blob URL for schema must contain /-/blob/",
        )
        self.assertNotIn("../epics/epic-01-core.md", body)
        self.assertNotIn("../../schema/platform.sysml", body)

    def test_github_relative_link_expansion(self):
        """Verifies GitHub issue creation receives expanded /blob/ URLs in body file."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue([self.feature_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.exists(self.gh_body_file), "gh body capture file should exist")
        with open(self.gh_body_file, "r", encoding="utf-8") as f:
            body = f.read()

        self.assertIn(
            "[Epic 01 Core](https://github.com/aerospace/uav-009-flight-control/blob/main/docs/epics/epic-01-core.md)",
            body,
            "GitHub blob URL must contain /blob/ and point to github.com",
        )
        self.assertIn(
            "[Platform Schema](https://github.com/aerospace/uav-009-flight-control/blob/main/schema/platform.sysml)",
            body,
            "GitHub blob URL for schema must contain /blob/",
        )
        self.assertNotIn("../epics/epic-01-core.md", body)
        self.assertNotIn("../../schema/platform.sysml", body)

    # -------------------------------------------------------------------------
    # 6. Missing Label Auto-Creation Across Providers
    # -------------------------------------------------------------------------

    def test_gitlab_label_creation_when_missing(self):
        """Verifies missing label triggers glab label create with #0366d6 color and description."""
        self._init_git_repo("git@gitlab.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"MOCK_GLAB_MISSING_LABEL": "1"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")
        self.assertIn("[GATE] Label 'feature' not found. Creating...", res.stdout)

        glab_calls = self._get_glab_calls()
        create_label_calls = [c for c in glab_calls if c[:2] == ["label", "create"]]
        self.assertEqual(len(create_label_calls), 1, "glab label create must be called")
        self.assertIn("feature", create_label_calls[0])
        self.assertIn("--color", create_label_calls[0])
        color_idx = create_label_calls[0].index("--color")
        self.assertEqual(create_label_calls[0][color_idx + 1], "#0366d6")

    def test_github_label_creation_when_missing(self):
        """Verifies missing label triggers gh label create with 0366d6 color and description."""
        self._init_git_repo("git@github.com:aerospace/uav-009-flight-control.git")
        res = self._run_create_issue(
            [self.feature_file, "feature", self.feature_title],
            extra_env={"MOCK_GH_MISSING_LABEL": "1"},
        )
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}\n{res.stdout}")
        self.assertIn("[GATE] Label 'feature' not found. Creating...", res.stdout)

        gh_calls = self._get_gh_calls()
        create_label_calls = [c for c in gh_calls if c[:2] == ["label", "create"]]
        self.assertEqual(len(create_label_calls), 1, "gh label create must be called")
        self.assertIn("feature", create_label_calls[0])
        self.assertIn("--color", create_label_calls[0])
        color_idx = create_label_calls[0].index("--color")
        self.assertEqual(create_label_calls[0][color_idx + 1], "0366d6")

    # -------------------------------------------------------------------------
    # 7. CLI Argument Validation Tests
    # -------------------------------------------------------------------------

    def test_zero_arguments_prints_usage(self):
        """Running with zero arguments exits non-zero, prints usage, and does not crash."""
        res = self._run_create_issue([])
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("Usage:", res.stderr)
        self.assertIn("<body-file> <label> <title> [repo]", res.stderr)

    def test_insufficient_arguments_prints_usage(self):
        """Running with 1 or 2 arguments exits non-zero and prints usage."""
        res1 = self._run_create_issue([self.feature_file])
        self.assertNotEqual(res1.returncode, 0)
        self.assertIn("Usage:", res1.stderr)

        res2 = self._run_create_issue([self.feature_file, "feature"])
        self.assertNotEqual(res2.returncode, 0)
        self.assertIn("Usage:", res2.stderr)

    def test_missing_body_file_fails(self):
        """Running with non-existent body file exits non-zero with FATAL message."""
        non_existent = os.path.join(self.workspace_dir, "does_not_exist.md")
        res = self._run_create_issue([non_existent, "feature", self.feature_title])
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("FATAL: Body file not found", res.stderr)


if __name__ == "__main__":
    unittest.main()

