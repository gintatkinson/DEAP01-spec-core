import os
import sys
import signal
import subprocess
import json
import unittest
import tempfile
import shutil
from unittest.mock import patch, MagicMock

# Add skills/spec-orchestrator/parity_auditor/src to sys.path
_src_dir = os.path.abspath(
    os.path.join(
        os.path.dirname(__file__),
        "..",
        "skills",
        "spec-orchestrator",
        "parity_auditor",
        "src",
    )
)
if _src_dir not in sys.path:
    sys.path.insert(0, _src_dir)

from parity_auditor.validators.sync_validator import (
    SyncValidator,
    parse_git_remote_url,
    get_git_remote_info,
    detect_tracker_provider,
    run_bounded_process,
    _fetch_gitlab_issues,
    _fetch_github_issues,
    _terminate_process_group,
)
from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.core.models import CodebaseRules, BacklogDirectories


class TestSyncValidatorRemoteParser(unittest.TestCase):
    def test_parse_https_gitlab_url(self):
        url = "https://gitlab.com/gintatkinson/DEAP01-spec-core.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "gintatkinson/DEAP01-spec-core")
        self.assertEqual(info["server_url"], "https://gitlab.com")
        self.assertEqual(info["host"], "gitlab.com")

    def test_parse_custom_domain_gitlab_url(self):
        url = "https://gitlab.internal.defense.gov/uas/uas-core.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "uas/uas-core")
        self.assertEqual(info["server_url"], "https://gitlab.internal.defense.gov")
        self.assertEqual(info["host"], "gitlab.internal.defense.gov")

    def test_parse_ssh_gitlab_url(self):
        url = "git@gitlab.com:defense-org/platform.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "defense-org/platform")
        self.assertEqual(info["server_url"], "https://gitlab.com")

    def test_parse_github_url(self):
        url = "https://github.com/gintatkinson/DEAP01-spec-core.git"
        info = parse_git_remote_url(url)
        self.assertFalse(info["is_gitlab"])
        self.assertEqual(info["project_path"], "gintatkinson/DEAP01-spec-core")
        self.assertEqual(info["server_url"], "https://github.com")

    def test_parse_empty_url(self):
        info = parse_git_remote_url("")
        self.assertFalse(info["is_gitlab"])
        self.assertIsNone(info["project_path"])


class TestSyncValidatorProviderDetection(unittest.TestCase):
    def test_cli_provider_override(self):
        prov = detect_tracker_provider(cli_provider="gitlab")
        self.assertEqual(prov, "gitlab")

        prov = detect_tracker_provider(cli_provider="github")
        self.assertEqual(prov, "github")

    def test_env_tracker_provider(self):
        with patch.dict(os.environ, {"TRACKER_PROVIDER": "gitlab"}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "gitlab")

        with patch.dict(os.environ, {"PROVIDER": "gitlab"}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "gitlab")

    def test_rules_gitlab_provider(self):
        rules = {"tracker_rules": {"provider": "gitlab"}}
        prov = detect_tracker_provider(rules=rules)
        self.assertEqual(prov, "gitlab")

    @patch.dict(os.environ, {"GITLAB_CI": "true"}, clear=True)
    def test_gitlab_ci_detection(self):
        prov = detect_tracker_provider()
        self.assertEqual(prov, "gitlab")

    @patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}, clear=True)
    def test_github_actions_detection(self):
        prov = detect_tracker_provider()
        self.assertEqual(prov, "github")

    @patch.dict(os.environ, {"JIRA_URL": "https://jira.corp"}, clear=True)
    def test_jira_env_detection(self):
        prov = detect_tracker_provider()
        self.assertEqual(prov, "jira")

    @patch("parity_auditor.validators.sync_validator.get_git_remote_info")
    def test_git_remote_detection(self, mock_remote):
        mock_remote.return_value = {"is_gitlab": True, "server_url": "https://gitlab.com"}
        with patch.dict(os.environ, {}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "gitlab")

    @patch("parity_auditor.validators.sync_validator.get_git_remote_info", return_value=None)
    def test_default_fallback_github(self, mock_remote):
        with patch.dict(os.environ, {}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "github")


class TestSyncValidatorSubprocessLifecycle(unittest.TestCase):
    @patch("subprocess.Popen")
    def test_run_bounded_process_success(self, mock_popen):
        mock_proc = MagicMock()
        mock_proc.communicate.return_value = ("output_data", "")
        mock_proc.returncode = 0
        mock_popen.return_value = mock_proc

        rc, stdout, stderr = run_bounded_process(["test_cmd"], cwd="/workspace", timeout=15.0)
        self.assertEqual(rc, 0)
        self.assertEqual(stdout, "output_data")
        self.assertEqual(stderr, "")
        mock_popen.assert_called_once_with(
            ["test_cmd"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd="/workspace",
            start_new_session=True,
        )

    @patch("os.killpg")
    @patch("os.getpgid", return_value=12345)
    @patch("subprocess.Popen")
    def test_run_bounded_process_timeout_kills_process_group(self, mock_popen, mock_getpgid, mock_killpg):
        mock_proc = MagicMock()
        mock_proc.pid = 12345
        mock_proc.poll.return_value = None
        mock_proc.communicate.side_effect = subprocess.TimeoutExpired(cmd=["test_cmd"], timeout=5.0)
        mock_popen.return_value = mock_proc

        with self.assertRaises(subprocess.TimeoutExpired):
            run_bounded_process(["test_cmd"], cwd="/workspace", timeout=5.0)

        mock_getpgid.assert_called_with(12345)
        mock_killpg.assert_called_with(12345, signal.SIGKILL)

    @patch("os.killpg")
    @patch("os.getpgid", return_value=54321)
    @patch("subprocess.Popen")
    def test_run_bounded_process_exception_kills_process_group(self, mock_popen, mock_getpgid, mock_killpg):
        mock_proc = MagicMock()
        mock_proc.pid = 54321
        mock_proc.poll.return_value = None
        mock_proc.communicate.side_effect = OSError("Pipe broken")
        mock_popen.return_value = mock_proc

        with self.assertRaises(OSError):
            run_bounded_process(["test_cmd"], cwd="/workspace", timeout=5.0)

        mock_killpg.assert_called_with(54321, signal.SIGKILL)

    @patch("os.killpg")
    @patch("os.getpgid", side_effect=ProcessLookupError)
    def test_terminate_process_group_handles_lookup_error(self, mock_getpgid, mock_killpg):
        mock_proc = MagicMock()
        mock_proc.pid = 99999
        mock_proc.poll.return_value = None

        # Should not raise exception
        _terminate_process_group(mock_proc)
        mock_proc.kill.assert_called_once()


class TestSyncValidatorErrorHandling(unittest.TestCase):
    @patch("shutil.which", return_value="/usr/bin/glab")
    @patch("parity_auditor.validators.sync_validator.run_bounded_process")
    def test_glab_cli_non_zero_exit_handled_gracefully(self, mock_run, mock_which):
        mock_run.return_value = (1, "", "fatal: project not found")
        with patch.dict(os.environ, {}, clear=True):
            issues = _fetch_gitlab_issues(workspace_dir="/workspace")
            self.assertEqual(issues, [])

    @patch("shutil.which", return_value="/usr/bin/gh")
    @patch("parity_auditor.validators.sync_validator.run_bounded_process")
    def test_gh_cli_non_zero_exit_handled_gracefully(self, mock_run, mock_which):
        mock_run.return_value = (1, "", "none of the git remotes configured point to github")
        with patch.dict(os.environ, {}, clear=True):
            issues = _fetch_github_issues(workspace_dir="/workspace")
            self.assertEqual(issues, [])

    @patch.dict(os.environ, {"OFFLINE": "1"}, clear=True)
    def test_offline_mode_returns_empty(self):
        issues_gl = _fetch_gitlab_issues(workspace_dir="/workspace")
        self.assertEqual(issues_gl, [])
        issues_gh = _fetch_github_issues(workspace_dir="/workspace")
        self.assertEqual(issues_gh, [])

    @patch("shutil.which", return_value="/usr/bin/gh")
    @patch("parity_auditor.validators.sync_validator.run_bounded_process")
    def test_timeout_handled_gracefully(self, mock_run, mock_which):
        mock_run.side_effect = subprocess.TimeoutExpired(cmd=["gh"], timeout=30.0)
        with patch.dict(os.environ, {}, clear=True):
            issues = _fetch_github_issues(workspace_dir="/workspace")
            self.assertEqual(issues, [])


class TestSyncValidatorGitLabPayloadMappingAndScopedLabels(unittest.TestCase):
    @patch("shutil.which", return_value="/usr/bin/glab")
    @patch("parity_auditor.validators.sync_validator.run_bounded_process")
    def test_gitlab_iid_mapped_to_number(self, mock_run, mock_which):
        raw_issues = [
            {"iid": 101, "title": "Epic 1: System Core", "state": "opened", "labels": ["type::epic"]},
            {"iid": 102, "title": "Feature 1: Flight Trajectory", "state": "opened", "labels": ["type::feature"]},
        ]
        mock_run.return_value = (0, json.dumps(raw_issues), "")
        with patch.dict(os.environ, {}, clear=True):
            issues = _fetch_gitlab_issues(workspace_dir="/workspace")
            self.assertEqual(len(issues), 2)
            self.assertEqual(issues[0]["number"], 101)
            self.assertEqual(issues[1]["number"], 102)

    def test_validator_with_scoped_labels_and_missing_specs(self):
        temp_dir = tempfile.mkdtemp()
        try:
            epics_dir = os.path.join(temp_dir, "docs", "epics")
            features_dir = os.path.join(temp_dir, "docs", "features")
            os.makedirs(epics_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            # Create one local epic file, but no feature file
            with open(os.path.join(epics_dir, "epic-01-system-core.md"), "w", encoding="utf-8") as f:
                f.write("# Epic 1: System Core\n")

            gitlab_issues = [
                {"iid": 1, "title": "Epic 1: System Core", "labels": ["type::epic"]},
                {"iid": 2, "title": "Feature 2: Sensor Ingestion", "labels": ["type::feature"]},
            ]

            repo = WorkspaceRepository(temp_dir)
            # Ensure not treated as upstream compiler repo with exemption
            with patch.object(repo, "is_upstream_compiler_repo", return_value=False):
                with patch("parity_auditor.validators.sync_validator.detect_tracker_provider", return_value="gitlab"):
                    with patch("parity_auditor.validators.sync_validator._fetch_gitlab_issues", return_value=gitlab_issues):
                        validator = SyncValidator()
                        findings = validator.validate(repo)

                        # Missing local specification for Feature 2
                        self.assertEqual(len(findings), 1)
                        self.assertEqual(findings[0].rule_id, "tracker-issue-without-local-specification")
                        self.assertIn("Issue #2", findings[0])
                        self.assertIn("Feature 2: Sensor Ingestion", findings[0])
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)

    def test_validator_with_scoped_labels_and_index_collision(self):
        temp_dir = tempfile.mkdtemp()
        try:
            epics_dir = os.path.join(temp_dir, "docs", "epics")
            features_dir = os.path.join(temp_dir, "docs", "features")
            os.makedirs(epics_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            # Create local feature with index 2 but title "Feature 2: Battery Management"
            with open(os.path.join(features_dir, "feat-02-battery.md"), "w", encoding="utf-8") as f:
                f.write("# Feature 2: Battery Management\n")

            # Tracker has Feature 2 as "Feature 2: Sensor Ingestion"
            gitlab_issues = [
                {"iid": 20, "title": "Feature 2: Sensor Ingestion", "labels": [{"name": "type::feature"}, {"name": "status::fixed-resolved"}]},
            ]

            repo = WorkspaceRepository(temp_dir)
            with patch.object(repo, "is_upstream_compiler_repo", return_value=False):
                with patch("parity_auditor.validators.sync_validator.detect_tracker_provider", return_value="gitlab"):
                    with patch("parity_auditor.validators.sync_validator._fetch_gitlab_issues", return_value=gitlab_issues):
                        validator = SyncValidator()
                        findings = validator.validate(repo)

                        # Should report missing spec for #20 and collision on index 2
                        collision_findings = [f for f in findings if f.rule_id == "spec-index-collides-with-tracker-issue"]
                        self.assertEqual(len(collision_findings), 1)
                        self.assertIn("Index collision detected", collision_findings[0])
                        self.assertIn("Issue #20", collision_findings[0])
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)


class TestSyncValidatorGitLabRestApi(unittest.TestCase):
    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_rest_api_fetch_with_token(self, mock_urlopen, mock_which):
        raw_issues = [
            {"iid": 301, "title": "Epic 1: Flight Core", "state": "opened", "labels": ["type::epic"]},
            {"iid": 302, "title": "Feature 1: Guidance Laws", "state": "opened", "labels": ["type::feature"]},
        ]
        resp = MagicMock()
        resp.status = 200
        resp.headers = {"X-Next-Page": ""}
        resp.read.return_value = json.dumps(raw_issues).encode("utf-8")
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        tracker_rules = {
            "server_url": "https://gitlab.com",
            "project_id": "defense/uas",
        }
        with patch.dict(os.environ, {"GITLAB_TOKEN": "glpat-secret-123"}, clear=True):
            issues = _fetch_gitlab_issues(tracker_rules=tracker_rules)
            self.assertEqual(len(issues), 2)
            self.assertEqual(issues[0]["number"], 301)
            self.assertEqual(issues[1]["number"], 302)

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_rest_api_pagination(self, mock_urlopen, mock_which):
        page1 = json.dumps([
            {"iid": 1, "title": "Epic 1", "state": "opened", "labels": ["type::epic"]},
        ]).encode("utf-8")
        page2 = json.dumps([
            {"iid": 2, "title": "Feature 2", "state": "opened", "labels": ["type::feature"]},
        ]).encode("utf-8")

        resp1 = MagicMock()
        resp1.status = 200
        resp1.headers = {"X-Next-Page": "2"}
        resp1.read.return_value = page1
        resp1.__enter__.return_value = resp1

        resp2 = MagicMock()
        resp2.status = 200
        resp2.headers = {"X-Next-Page": ""}
        resp2.read.return_value = page2
        resp2.__enter__.return_value = resp2

        mock_urlopen.side_effect = [resp1, resp2]

        tracker_rules = {
            "server_url": "https://gitlab.example.com",
            "project_id": "org/proj",
        }
        with patch.dict(os.environ, {"CI_JOB_TOKEN": "job-token-456"}, clear=True):
            issues = _fetch_gitlab_issues(tracker_rules=tracker_rules)
            self.assertEqual(len(issues), 2)
            self.assertEqual(issues[0]["number"], 1)
            self.assertEqual(issues[1]["number"], 2)

    @patch("shutil.which", return_value=None)
    def test_gitlab_missing_token_returns_empty(self, mock_which):
        tracker_rules = {
            "server_url": "https://gitlab.com",
            "project_id": "org/proj",
        }
        with patch.dict(os.environ, {}, clear=True):
            with patch("netrc.netrc", side_effect=FileNotFoundError):
                issues = _fetch_gitlab_issues(tracker_rules=tracker_rules)
                self.assertEqual(issues, [])

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_api_network_error_handled(self, mock_urlopen, mock_which):
        mock_urlopen.side_effect = Exception("Connection refused")
        tracker_rules = {
            "server_url": "https://gitlab.com",
            "project_id": "org/proj",
        }
        with patch.dict(os.environ, {"GITLAB_TOKEN": "glpat-token"}, clear=True):
            issues = _fetch_gitlab_issues(tracker_rules=tracker_rules)
            self.assertEqual(issues, [])


if __name__ == "__main__":
    unittest.main()
