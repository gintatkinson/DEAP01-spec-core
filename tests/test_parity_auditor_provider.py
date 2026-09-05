import os
import sys
import json
import unittest
from unittest.mock import patch, MagicMock

# Add skills/spec-orchestrator/parity_auditor/src to sys.path
_src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "skills", "spec-orchestrator", "parity_auditor", "src"))
if _src_dir not in sys.path:
    sys.path.insert(0, _src_dir)

from parity_auditor.cli import (
    parse_git_remote_url,
    get_git_remote_info,
    detect_tracker_provider,
    get_open_feature_issues,
    _fetch_gitlab_issues,
    _fetch_github_issues,
)


class TestParityAuditorRemoteParser(unittest.TestCase):
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


class TestParityAuditorProviderDetection(unittest.TestCase):
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

    @patch("parity_auditor.cli.get_git_remote_info")
    def test_git_remote_detection(self, mock_remote):
        mock_remote.return_value = {"is_gitlab": True, "server_url": "https://gitlab.com"}
        with patch.dict(os.environ, {}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "gitlab")

    @patch("parity_auditor.cli.get_git_remote_info", return_value=None)
    def test_default_fallback_github(self, mock_remote):
        with patch.dict(os.environ, {}, clear=True):
            prov = detect_tracker_provider()
            self.assertEqual(prov, "github")


class TestParityAuditorGitHubIssues(unittest.TestCase):
    @patch("shutil.which", return_value="/usr/bin/gh")
    @patch("subprocess.run")
    def test_github_fetch_success_and_filtering(self, mock_run, mock_which):
        raw_issues = [
            {"number": 101, "title": "Implement Flight Control Laws"},
            {"number": 102, "title": "Tooling Defect: Crash on startup"},
            {"number": 103, "title": "Repro bug in sensor mapping"},
            {"number": 104, "title": "Add Actuator Interface Binding"},
        ]
        mock_result = MagicMock()
        mock_result.returncode = 0
        mock_result.stdout = json.dumps(raw_issues)
        mock_run.return_value = mock_result

        issues = _fetch_github_issues()
        self.assertIsNotNone(issues)
        self.assertEqual(len(issues), 2)
        self.assertEqual(issues[0]["number"], 101)
        self.assertEqual(issues[1]["number"], 104)

    @patch("shutil.which", return_value="/usr/bin/gh")
    @patch("subprocess.run")
    def test_github_fetch_failure_returns_none(self, mock_run, mock_which):
        mock_result = MagicMock()
        mock_result.returncode = 1
        mock_result.stderr = "none of the git remotes configured point to github"
        mock_run.return_value = mock_result

        issues = _fetch_github_issues()
        self.assertIsNone(issues)

    @patch.dict(os.environ, {"OFFLINE": "1"})
    def test_github_offline_returns_none(self):
        issues = _fetch_github_issues()
        self.assertIsNone(issues)


class TestParityAuditorGitLabIssues(unittest.TestCase):
    @patch("shutil.which")
    @patch("subprocess.run")
    def test_gitlab_glab_cli_fetch(self, mock_run, mock_which):
        def which_side_effect(cmd):
            if cmd == "glab":
                return "/usr/bin/glab"
            return None
        mock_which.side_effect = which_side_effect

        raw_issues = [
            {"iid": 201, "title": "Feature 1", "state": "opened", "labels": ["type::feature"]},
            {"iid": 202, "title": "Bug fix for telemetry", "state": "opened", "labels": ["type::feature"]},
            {"iid": 203, "title": "Feature 2", "state": "closed", "labels": ["type::feature"]},
            {"iid": 204, "title": "Feature 3", "state": "opened", "labels": ["feature"]},
        ]
        mock_result = MagicMock()
        mock_result.returncode = 0
        mock_result.stdout = json.dumps(raw_issues)
        mock_run.return_value = mock_result

        issues = _fetch_gitlab_issues()
        self.assertIsNotNone(issues)
        self.assertEqual(len(issues), 2)
        self.assertEqual(issues[0]["number"], 201)
        self.assertEqual(issues[1]["number"], 204)

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_rest_api_fetch_with_token(self, mock_urlopen, mock_which):
        raw_issues = [
            {"iid": 301, "title": "Sensor Fusion Feature", "state": "opened", "labels": ["type::feature"]},
            {"iid": 302, "title": "Tooling bug fix", "state": "opened", "labels": ["type::feature"]},
        ]
        resp = MagicMock()
        resp.status = 200
        resp.headers = {"X-Next-Page": ""}
        resp.read.return_value = json.dumps(raw_issues).encode("utf-8")
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        rules = {
            "tracker_rules": {
                "server_url": "https://gitlab.com",
                "project_id": "defense/uas",
            }
        }
        with patch.dict(os.environ, {"GITLAB_TOKEN": "glpat-secret-123"}, clear=True):
            issues = _fetch_gitlab_issues(rules=rules)
            self.assertIsNotNone(issues)
            self.assertEqual(len(issues), 1)
            self.assertEqual(issues[0]["number"], 301)

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_rest_api_pagination(self, mock_urlopen, mock_which):
        page1 = json.dumps([
            {"iid": 1, "title": "Feat 1", "state": "opened", "labels": ["type::feature"]},
        ]).encode("utf-8")
        page2 = json.dumps([
            {"iid": 2, "title": "Feat 2", "state": "opened", "labels": ["type::feature"]},
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

        rules = {
            "tracker_rules": {
                "server_url": "https://gitlab.example.com",
                "project_id": "org/proj",
            }
        }
        with patch.dict(os.environ, {"CI_JOB_TOKEN": "job-token-456"}, clear=True):
            issues = _fetch_gitlab_issues(rules=rules)
            self.assertIsNotNone(issues)
            self.assertEqual(len(issues), 2)
            self.assertEqual(issues[0]["number"], 1)
            self.assertEqual(issues[1]["number"], 2)

    @patch("shutil.which", return_value=None)
    def test_gitlab_missing_token_returns_none(self, mock_which):
        rules = {
            "tracker_rules": {
                "server_url": "https://gitlab.com",
                "project_id": "org/proj",
            }
        }
        with patch.dict(os.environ, {}, clear=True):
            with patch("netrc.netrc", side_effect=FileNotFoundError):
                issues = _fetch_gitlab_issues(rules=rules)
                self.assertIsNone(issues)

    @patch.dict(os.environ, {"OFFLINE": "1"})
    def test_gitlab_offline_mode(self):
        issues = _fetch_gitlab_issues()
        self.assertIsNone(issues)

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_gitlab_api_network_error_handled(self, mock_urlopen, mock_which):
        mock_urlopen.side_effect = Exception("Connection refused")
        rules = {
            "tracker_rules": {
                "server_url": "https://gitlab.com",
                "project_id": "org/proj",
            }
        }
        with patch.dict(os.environ, {"GITLAB_TOKEN": "glpat-token"}, clear=True):
            issues = _fetch_gitlab_issues(rules=rules)
            self.assertIsNone(issues)

    @patch("parity_auditor.cli._fetch_gitlab_issues")
    def test_get_open_feature_issues_dispatches_to_gitlab(self, mock_fetch_gl):
        mock_fetch_gl.return_value = [{"number": 99, "title": "GitLab Feature"}]
        issues = get_open_feature_issues(provider="gitlab")
        self.assertEqual(len(issues), 1)
        self.assertEqual(issues[0]["number"], 99)
        mock_fetch_gl.assert_called_once()


if __name__ == "__main__":
    unittest.main()
