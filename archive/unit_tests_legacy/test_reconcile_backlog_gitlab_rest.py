#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com

"""
Regression tests for GitLab REST API v4 provider adapter in reconcile_backlog.py (Issue #220).

Asserts that:
1. When `glab` CLI binary is absent from PATH, reconcile_backlog uses direct GitLab REST API v4 backend via urllib.request.
2. Token resolution correctly reads from GITLAB_TOKEN, GL_TOKEN, CI_JOB_TOKEN, and .netrc.
3. Project ID / path resolution correctly parses git remote URLs, --project, and --gitlab-group.
4. Issue listing, description update, title update, label addition, and issue commenting execute cleanly via REST API.
5. Graceful fallback to offline/local specification mode occurs when token or network connectivity is missing, without fatal crashes.
"""

import io
import os
import sys
import json
import ssl
import urllib.request
import urllib.error
import tempfile
import unittest
from unittest.mock import patch, MagicMock

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    GitLabV4Provider,
    create_tracker_provider,
    detect_tracker_provider,
    load_codebase_rules,
    sync_issue_body_to_tracker,
    sync_issue_title_to_tracker,
    apply_structural_label,
    resolve_issue_on_tracker,
    get_all_issues,
    DEFAULT_GITLAB_TRACKER_RULES,
)


class MockHTTPResponse:
    def __init__(self, status=200, body=b"", headers=None):
        self.status = status
        self._body = body if isinstance(body, bytes) else body.encode("utf-8")
        self.headers = headers or {}

    def read(self, *args, **kwargs):
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass


class TestGitLabV4ProviderAuthAndResolution(unittest.TestCase):
    def setUp(self):
        self.env_patcher = patch.dict(os.environ, {}, clear=True)
        self.env_patcher.start()

    def tearDown(self):
        self.env_patcher.stop()

    def test_resolve_token_from_gitlab_token(self):
        os.environ["GITLAB_TOKEN"] = "glpat-secret-token-123"
        provider = GitLabV4Provider(server_url="https://gitlab.example.com", project_id="org/repo")
        self.assertEqual(provider.token, "glpat-secret-token-123")
        self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    def test_resolve_token_from_gl_token(self):
        os.environ["GL_TOKEN"] = "glpat-alternate-token-456"
        provider = GitLabV4Provider(server_url="https://gitlab.example.com", project_id="org/repo")
        self.assertEqual(provider.token, "glpat-alternate-token-456")
        self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    def test_resolve_token_from_ci_job_token(self):
        os.environ["CI_JOB_TOKEN"] = "job-token-789"
        provider = GitLabV4Provider(server_url="https://gitlab.example.com", project_id="org/repo")
        self.assertEqual(provider.token, "job-token-789")
        self.assertEqual(provider.token_type, "JOB-TOKEN")

    @patch("netrc.netrc")
    def test_resolve_token_from_netrc(self, mock_netrc_class):
        mock_netrc_inst = MagicMock()
        mock_netrc_inst.authenticators.return_value = ("oauth2", None, "netrc-secret-token")
        mock_netrc_class.return_value = mock_netrc_inst

        provider = GitLabV4Provider(server_url="https://gitlab.custom.org", project_id="group/proj")
        self.assertEqual(provider.token, "netrc-secret-token")
        self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    def test_resolve_project_id_from_ci_project_path(self):
        os.environ["CI_PROJECT_PATH"] = "aviation/uas-control-system"
        provider = GitLabV4Provider(server_url="https://gitlab.com")
        self.assertEqual(provider.raw_project_id, "aviation/uas-control-system")
        self.assertEqual(provider.project_id_encoded, "aviation%2Fuas-control-system")

    def test_resolve_project_id_numeric(self):
        provider = GitLabV4Provider(server_url="https://gitlab.com", project_id="98765")
        self.assertEqual(provider.raw_project_id, "98765")
        self.assertEqual(provider.project_id_encoded, "98765")

    def test_resolve_project_id_from_group_and_project_env(self):
        os.environ["GITLAB_GROUP"] = "deepmind"
        os.environ["GITLAB_PROJECT_NAME"] = "spec-compiler"
        provider = GitLabV4Provider(server_url="https://gitlab.com")
        self.assertEqual(provider.raw_project_id, "deepmind/spec-compiler")
        self.assertEqual(provider.project_id_encoded, "deepmind%2Fspec-compiler")

    def test_create_tracker_provider_with_cli_group_and_project(self):
        rules = {"tracker_rules": {"provider": "gitlab"}}
        provider = create_tracker_provider(
            "gitlab",
            rules=rules,
            cli_gitlab_url="https://gitlab.corp.internal",
            cli_project="core-compiler",
            cli_gitlab_group="engineering/embedded",
            cli_token="cli-passed-token-999"
        )
        self.assertIsInstance(provider, GitLabV4Provider)
        self.assertEqual(provider.server_url, "https://gitlab.corp.internal")
        self.assertEqual(provider.raw_project_id, "engineering/embedded/core-compiler")
        self.assertEqual(provider.project_id_encoded, "engineering%2Fembedded%2Fcore-compiler")
        self.assertEqual(provider.token, "cli-passed-token-999")


class TestGitLabV4ProviderRestOperations(unittest.TestCase):
    def setUp(self):
        self.provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="mygroup/myproject",
            token="glpat-test-token",
        )

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_list_issues_rest_pagination_and_normalization(self, mock_urlopen, mock_which):
        # Assert glab CLI is absent
        self.assertIsNone(mock_which("glab"))

        # Page 1 response (2 items, X-Next-Page: 2)
        page1_data = [
            {
                "id": 1001,
                "iid": 42,
                "title": "Feature 42: Flight Guidance Controller",
                "description": "# Flight Guidance Spec\n\nAC-01 Given...",
                "state": "opened",
                "labels": ["type::feature"],
            }
        ]
        # Page 2 response (1 item, X-Next-Page: "")
        page2_data = [
            {
                "id": 1002,
                "iid": 43,
                "title": "Epic 01: Core Systems",
                "description": "# Core Systems Epic",
                "state": "closed",
                "labels": ["type::epic", "status::fixed-resolved"],
            }
        ]

        resp1 = MockHTTPResponse(
            status=200,
            body=json.dumps(page1_data),
            headers={"X-Next-Page": "2"}
        )
        resp2 = MockHTTPResponse(
            status=200,
            body=json.dumps(page2_data),
            headers={"X-Next-Page": ""}
        )
        mock_urlopen.side_effect = [resp1, resp2]

        issues = self.provider.list_issues()
        self.assertEqual(len(issues), 2)

        # Check normalization
        self.assertEqual(issues[0]["number"], 42)
        self.assertEqual(issues[0]["issue_id"], 42)
        self.assertEqual(issues[0]["state"], "OPENED")
        self.assertEqual(issues[0]["title"], "Feature 42: Flight Guidance Controller")
        self.assertEqual(issues[0]["body"], "# Flight Guidance Spec\n\nAC-01 Given...")

        self.assertEqual(issues[1]["number"], 43)
        self.assertEqual(issues[1]["issue_id"], 43)
        self.assertEqual(issues[1]["state"], "CLOSED")

        # Verify urllib.request.urlopen was called twice for REST pagination
        self.assertEqual(mock_urlopen.call_count, 2)
        first_req = mock_urlopen.call_args_list[0][0][0]
        self.assertIn("projects/mygroup%2Fmyproject/issues", first_req.full_url)
        self.assertEqual(first_req.headers.get("Private-token"), "glpat-test-token")

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_edit_issue_rest(self, mock_urlopen, mock_which):
        self.assertIsNone(mock_which("glab"))
        mock_urlopen.return_value = MockHTTPResponse(status=200, body=json.dumps({"iid": 42, "description": "Updated"}))

        result = self.provider.edit_issue(42, "Updated body markdown content")
        self.assertTrue(result)

        self.assertEqual(mock_urlopen.call_count, 1)
        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_method(), "PUT")
        self.assertIn("projects/mygroup%2Fmyproject/issues/42", req.full_url)
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload, {"description": "Updated body markdown content"})

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_edit_issue_title_rest(self, mock_urlopen, mock_which):
        self.assertIsNone(mock_which("glab"))
        mock_urlopen.return_value = MockHTTPResponse(status=200, body=json.dumps({"iid": 42, "title": "New Title"}))

        result = self.provider.edit_issue_title(42, "New Title")
        self.assertTrue(result)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_method(), "PUT")
        self.assertIn("projects/mygroup%2Fmyproject/issues/42", req.full_url)
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload, {"title": "New Title"})

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_add_label_rest(self, mock_urlopen, mock_which):
        self.assertIsNone(mock_which("glab"))
        mock_urlopen.return_value = MockHTTPResponse(status=200, body=json.dumps({"iid": 42, "labels": ["type::feature"]}))

        result = self.provider.add_label(42, "type::feature")
        self.assertTrue(result)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_method(), "PUT")
        self.assertIn("projects/mygroup%2Fmyproject/issues/42", req.full_url)
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload, {"add_labels": "type::feature"})

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_comment_issue_rest(self, mock_urlopen, mock_which):
        self.assertIsNone(mock_which("glab"))
        mock_urlopen.return_value = MockHTTPResponse(status=201, body=json.dumps({"id": 555, "body": "Resolved comment"}))

        result = self.provider.comment_issue(42, "Resolved comment note")
        self.assertTrue(result)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_method(), "POST")
        self.assertIn("projects/mygroup%2Fmyproject/issues/42/notes", req.full_url)
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload, {"body": "Resolved comment note"})

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_create_label_rest_and_conflict_handling(self, mock_urlopen, mock_which):
        self.assertIsNone(mock_which("glab"))
        # 1. Successful label creation (201)
        mock_urlopen.return_value = MockHTTPResponse(status=201, body=json.dumps({"name": "status::fixed-resolved"}))
        res1 = self.provider.create_label("status::fixed-resolved", description="Resolved label", color="#0E8A16")
        self.assertTrue(res1)

        # 2. 409 Conflict (Label already exists)
        err = urllib.error.HTTPError(
            url="https://gitlab.example.com/api/v4/projects/mygroup%2Fmyproject/labels",
            code=409,
            msg="Conflict",
            hdrs={},
            fp=io.BytesIO(b'{"message": "Label already exists"}'),
        )
        mock_urlopen.side_effect = err
        res2 = self.provider.create_label("status::fixed-resolved")
        self.assertTrue(res2)


class TestGitLabOfflineAndErrorGracefulHandling(unittest.TestCase):
    def test_offline_mode_returns_empty_without_network_calls(self):
        provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="mygroup/myproject",
            token="glpat-test-token",
            offline=True
        )
        with patch("urllib.request.urlopen") as mock_urlopen, patch("subprocess.run") as mock_run:
            issues = provider.list_issues()
            self.assertEqual(issues, [])
            self.assertEqual(mock_urlopen.call_count, 0)
            self.assertEqual(mock_run.call_count, 0)

            # Mutating operations return False/None safely
            self.assertFalse(provider.edit_issue(42, "body"))
            self.assertFalse(provider.add_label(42, "type::feature"))
            self.assertFalse(provider.comment_issue(42, "note"))
            self.assertIsNone(provider.create_issue("title", "desc"))

    @patch("shutil.which", return_value=None)
    def test_unauthenticated_without_glab_returns_empty_with_notice(self, mock_which):
        # No token in environment or args, glab not on PATH
        provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="mygroup/myproject",
            token=None
        )
        with patch("urllib.request.urlopen") as mock_urlopen, patch("subprocess.run") as mock_run:
            issues = provider.list_issues()
            self.assertEqual(issues, [])
            self.assertEqual(mock_urlopen.call_count, 0)
            self.assertEqual(mock_run.call_count, 0)

    @patch("shutil.which", return_value=None)
    @patch("urllib.request.urlopen")
    def test_network_transport_failure_recovers_gracefully(self, mock_urlopen, mock_which):
        provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="mygroup/myproject",
            token="glpat-token",
            max_retries=1
        )
        mock_urlopen.side_effect = urllib.error.URLError(reason="Connection refused")
        issues = provider.list_issues()
        self.assertEqual(issues, [])


class TestReconcileBacklogGitLabIntegration(unittest.TestCase):
    def setUp(self):
        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.example.com",
                "project_id": "aviation/flight-control",
                "labels": {
                    "epic": "type::epic",
                    "feature": "type::feature",
                    "user_story": "type::user-story",
                    "use_case": "type::use-case",
                    "resolved": "status::fixed-resolved",
                }
            }
        }

    @patch("shutil.which", return_value=None)
    def test_sync_issue_body_and_resolve_via_gitlab_provider(self, mock_which):
        mock_provider = MagicMock(spec=GitLabV4Provider)
        mock_provider.edit_issue.return_value = True
        mock_provider.edit_issue_title.return_value = True
        mock_provider.add_label.return_value = True
        mock_provider.comment_issue.return_value = True
        mock_provider.create_label.return_value = True

        spec_content = (
            "---\n"
            "issue_id: 101\n"
            "title: Flight Guidance System\n"
            "type: feature\n"
            "---\n\n"
            "# Feature 101: Flight Guidance System\n\n"
            "## Acceptance Criteria\n"
            "Given aircraft state, When mode engaged, Then guidance active.\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            # Sync issue body to GitLab tracker
            sync_issue_body_to_tracker(
                issue_num=101,
                filepath=temp_path,
                issue_type="Feature",
                rules=self.gitlab_rules,
                provider_adapter=mock_provider
            )

            mock_provider.edit_issue.assert_called_once()
            mock_provider.add_label.assert_called_with(101, "type::feature")

            # Resolve issue on GitLab tracker
            resolve_issue_on_tracker(
                issue_num=101,
                comment="Resolved. All acceptance criteria verified.",
                rules=self.gitlab_rules,
                provider_adapter=mock_provider
            )

            mock_provider.add_label.assert_called_with(101, "status::fixed-resolved")
            mock_provider.comment_issue.assert_called_with(101, "Resolved. All acceptance criteria verified.")
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)


if __name__ == "__main__":
    unittest.main()
