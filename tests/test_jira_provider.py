import os
import sys
import json
import base64
import unittest
import netrc
import urllib.error
from unittest.mock import patch, MagicMock

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    JiraV2V3Provider,
    JiraRESTProvider,
    GitHubCLIProvider,
    detect_tracker_provider,
    load_codebase_rules,
    get_structural_label,
    get_resolved_label,
    create_tracker_provider,
    DEFAULT_JIRA_TRACKER_RULES,
    DEFAULT_JIRA_STRUCTURAL_LABELS,
)


class TestJiraAuthResolution(unittest.TestCase):
    def test_jira_cloud_basic_auth_header(self):
        """Verify Jira Cloud Basic Auth header encoding (email:API_TOKEN base64)."""
        provider = JiraV2V3Provider(
            server_url="https://company.atlassian.net",
            project_key="DEAP",
            email="developer@example.com",
            token="cloud_api_token_xyz123",
            token_type="Basic",
        )
        self.assertEqual(provider.server_url, "https://company.atlassian.net")
        self.assertEqual(provider.project_key, "DEAP")
        self.assertEqual(provider.email, "developer@example.com")
        self.assertEqual(provider.token, "cloud_api_token_xyz123")
        self.assertEqual(provider.token_type, "Basic")

        headers = provider._build_headers()
        expected_creds = base64.b64encode(b"developer@example.com:cloud_api_token_xyz123").decode("utf-8")
        self.assertEqual(headers["Authorization"], f"Basic {expected_creds}")
        self.assertEqual(headers["Accept"], "application/json")

    def test_jira_datacenter_bearer_pat_header(self):
        """Verify Jira Data Center / Server Bearer PAT header resolution."""
        provider = JiraV2V3Provider(
            server_url="https://jira.datacenter.internal.defense.gov",
            project_key="SAFETY",
            token="pat_secure_token_999",
            token_type="Bearer",
        )
        self.assertEqual(provider.server_url, "https://jira.datacenter.internal.defense.gov")
        self.assertEqual(provider.project_key, "SAFETY")
        self.assertEqual(provider.token, "pat_secure_token_999")
        self.assertEqual(provider.token_type, "Bearer")

        headers = provider._build_headers()
        self.assertEqual(headers["Authorization"], "Bearer pat_secure_token_999")
        self.assertEqual(headers["Accept"], "application/json")

    @patch.dict(os.environ, {
        "JIRA_SERVER_URL": "https://jira.datacenter.corp",
        "JIRA_PROJECT_KEY": "PROJ",
        "JIRA_PAT": "pat_from_env_123"
    }, clear=True)
    def test_jira_pat_env_resolution(self):
        provider = JiraV2V3Provider()
        self.assertEqual(provider.server_url, "https://jira.datacenter.corp")
        self.assertEqual(provider.project_key, "PROJ")
        self.assertEqual(provider.token, "pat_from_env_123")
        self.assertEqual(provider.token_type, "Bearer")
        headers = provider._build_headers()
        self.assertEqual(headers["Authorization"], "Bearer pat_from_env_123")

    @patch.dict(os.environ, {
        "JIRA_URL": "https://jira.cloud.corp",
        "JIRA_EMAIL": "agent@corp.com",
        "JIRA_API_TOKEN": "api_token_from_env"
    }, clear=True)
    def test_jira_cloud_env_resolution(self):
        provider = JiraV2V3Provider()
        self.assertEqual(provider.server_url, "https://jira.cloud.corp")
        self.assertEqual(provider.email, "agent@corp.com")
        self.assertEqual(provider.token, "api_token_from_env")
        self.assertEqual(provider.token_type, "Basic")
        headers = provider._build_headers()
        expected = base64.b64encode(b"agent@corp.com:api_token_from_env").decode("utf-8")
        self.assertEqual(headers["Authorization"], f"Basic {expected}")

    @patch("netrc.netrc")
    def test_jira_netrc_token_resolution(self, mock_netrc_class):
        """Verify .netrc credential lookup when explicit token is absent."""
        with patch.dict(os.environ, {}, clear=True):
            mock_netrc_inst = MagicMock()
            mock_netrc_inst.authenticators.return_value = ("netrc_agent", "account", "netrc_secret_token_abc")
            mock_netrc_class.return_value = mock_netrc_inst

            provider = JiraV2V3Provider(
                server_url="https://jira.custom.domain.net",
                project_key="NETRC_PROJ",
            )
            self.assertEqual(provider.email, "netrc_agent")
            self.assertEqual(provider.token, "netrc_secret_token_abc")
            self.assertEqual(provider.token_type, "Basic")

            headers = provider._build_headers()
            expected = base64.b64encode(b"netrc_agent:netrc_secret_token_abc").decode("utf-8")
            self.assertEqual(headers["Authorization"], f"Basic {expected}")
            mock_netrc_inst.authenticators.assert_called_once_with("jira.custom.domain.net")

    @patch("netrc.netrc", side_effect=FileNotFoundError("~/.netrc not found"))
    def test_jira_netrc_missing_fallback(self, mock_netrc_class):
        with patch.dict(os.environ, {}, clear=True):
            provider = JiraV2V3Provider(
                server_url="https://jira.example.com",
                project_key="TEST",
            )
            self.assertIsNone(provider.token)
            self.assertIsNone(provider.email)


class TestJiraApiOperations(unittest.TestCase):
    def setUp(self):
        self.provider = JiraV2V3Provider(
            server_url="https://jira.example.com",
            project_key="DEAP",
            token="test-pat-token",
            token_type="Bearer",
        )

    @patch("urllib.request.urlopen")
    def test_jql_search_pagination(self, mock_urlopen):
        """Verify JQL search with pagination returns aggregated issues matching DEAP tracker schema."""
        page1_data = json.dumps({
            "startAt": 0,
            "maxResults": 2,
            "total": 3,
            "issues": [
                {
                    "id": "10001",
                    "key": "DEAP-1",
                    "fields": {
                        "summary": "Epic 1: UAS Safety Core",
                        "description": "Epic specification body",
                        "status": {"name": "In Progress"},
                        "labels": ["type::epic"],
                        "issuetype": {"name": "Epic"}
                    }
                },
                {
                    "id": "10002",
                    "key": "DEAP-2",
                    "fields": {
                        "summary": "Feature 1: Geofence Enforcement",
                        "description": "Feature specification body",
                        "status": {"name": "Done"},
                        "labels": ["type::feature"],
                        "issuetype": {"name": "Feature"}
                    }
                }
            ]
        }).encode("utf-8")

        page2_data = json.dumps({
            "startAt": 2,
            "maxResults": 2,
            "total": 3,
            "issues": [
                {
                    "id": "10003",
                    "key": "DEAP-3",
                    "fields": {
                        "summary": "User Story 1: Breach Alert",
                        "description": "Story specification body",
                        "status": {"name": "Done"},
                        "labels": ["type::user-story"],
                        "issuetype": {"name": "Story"}
                    }
                }
            ]
        }).encode("utf-8")

        resp1 = MagicMock()
        resp1.status = 200
        resp1.headers = {}
        resp1.read.return_value = page1_data
        resp1.__enter__.return_value = resp1

        resp2 = MagicMock()
        resp2.status = 200
        resp2.headers = {}
        resp2.read.return_value = page2_data
        resp2.__enter__.return_value = resp2

        mock_urlopen.side_effect = [resp1, resp2]

        issues = self.provider.list_issues()
        self.assertEqual(len(issues), 3)

        # Verify DEAP tracker schema compliance
        for issue in issues:
            for required_key in ("id", "key", "number", "title", "body", "state", "labels", "issue_type"):
                self.assertIn(required_key, issue)

        self.assertEqual(issues[0]["key"], "DEAP-1")
        self.assertEqual(issues[0]["title"], "Epic 1: UAS Safety Core")
        self.assertEqual(issues[0]["state"], "IN PROGRESS")
        self.assertEqual(issues[0]["issue_type"], "Epic")
        self.assertEqual(issues[0]["labels"], ["type::epic"])

        self.assertEqual(issues[1]["key"], "DEAP-2")
        self.assertEqual(issues[1]["state"], "DONE")
        self.assertEqual(issues[1]["issue_type"], "Feature")

        self.assertEqual(issues[2]["key"], "DEAP-3")
        self.assertEqual(issues[2]["state"], "DONE")
        self.assertEqual(issues[2]["issue_type"], "Story")

    @patch("urllib.request.urlopen")
    def test_get_issue_success(self, mock_urlopen):
        """Verify get_issue fetches a single issue and converts to DEAP schema."""
        issue_data = json.dumps({
            "id": "10005",
            "key": "DEAP-5",
            "fields": {
                "summary": "Use Case 1: Autonomous RTH",
                "description": "RTH spec content",
                "status": {"name": "Done"},
                "labels": ["type::use-case"],
                "issuetype": {"name": "Use Case"}
            }
        }).encode("utf-8")

        resp = MagicMock()
        resp.status = 200
        resp.headers = {}
        resp.read.return_value = issue_data
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        issue = self.provider.get_issue("DEAP-5")
        self.assertIsNotNone(issue)
        self.assertEqual(issue["key"], "DEAP-5")
        self.assertEqual(issue["title"], "Use Case 1: Autonomous RTH")
        self.assertEqual(issue["state"], "DONE")
        self.assertEqual(issue["issue_type"], "Use Case")

    @patch("urllib.request.urlopen")
    def test_create_issue_success(self, mock_urlopen):
        """Verify create_issue sends POST /rest/api/2/issue with correct fields."""
        resp_data = json.dumps({
            "id": "10010",
            "key": "DEAP-10",
            "self": "https://jira.example.com/rest/api/2/issue/10010"
        }).encode("utf-8")

        resp = MagicMock()
        resp.status = 201
        resp.headers = {}
        resp.read.return_value = resp_data
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        created = self.provider.create_issue(
            title="Feature 2: Dynamic Obstacle Avoidance",
            body="Obstacle avoidance body text",
            labels=["type::feature"],
            issue_type="Feature",
            parent_key="DEAP-1",
            custom_fields={"customfield_9999": "safety_level_4"},
        )
        self.assertIsNotNone(created)
        self.assertEqual(created["key"], "DEAP-10")
        self.assertEqual(created["number"], "DEAP-10")

        # Verify request parameters
        call_args = mock_urlopen.call_args
        req = call_args[0][0]
        self.assertEqual(req.method, "POST")
        self.assertTrue(req.full_url.endswith("/rest/api/2/issue"))
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["fields"]["project"]["key"], "DEAP")
        self.assertEqual(payload["fields"]["summary"], "Feature 2: Dynamic Obstacle Avoidance")
        self.assertEqual(payload["fields"]["description"], "Obstacle avoidance body text")
        self.assertEqual(payload["fields"]["issuetype"]["name"], "Feature")
        self.assertEqual(payload["fields"]["parent"]["key"], "DEAP-1")
        self.assertEqual(payload["fields"]["customfield_9999"], "safety_level_4")

    @patch("urllib.request.urlopen")
    def test_edit_issue_success(self, mock_urlopen):
        """Verify edit_issue sends PUT /rest/api/2/issue/{key}."""
        resp = MagicMock()
        resp.status = 204
        resp.headers = {}
        resp.read.return_value = b""
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.edit_issue(
            "DEAP-10",
            title="Updated Title",
            body="Updated Body",
            labels=["type::feature", "priority::high"],
            custom_fields={"customfield_1001": "critical"}
        )
        self.assertTrue(success)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.method, "PUT")
        self.assertTrue(req.full_url.endswith("/rest/api/2/issue/DEAP-10"))
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["fields"]["summary"], "Updated Title")
        self.assertEqual(payload["fields"]["description"], "Updated Body")
        self.assertEqual(payload["fields"]["labels"], ["type::feature", "priority::high"])
        self.assertEqual(payload["fields"]["customfield_1001"], "critical")

    @patch("urllib.request.urlopen")
    def test_add_label_success(self, mock_urlopen):
        """Verify add_label updates Jira issue labels using atomic update array."""
        resp = MagicMock()
        resp.status = 204
        resp.headers = {}
        resp.read.return_value = b""
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.add_label("DEAP-10", "status::ready-for-review")
        self.assertTrue(success)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.method, "PUT")
        self.assertTrue(req.full_url.endswith("/rest/api/2/issue/DEAP-10"))
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["update"]["labels"], [{"add": "status::ready-for-review"}])

    @patch("urllib.request.urlopen")
    def test_comment_issue_success(self, mock_urlopen):
        """Verify comment_issue posts comment body to /rest/api/2/issue/{key}/comment."""
        resp = MagicMock()
        resp.status = 201
        resp.headers = {}
        resp.read.return_value = b'{"id": "20001", "body": "Reconciliation verified"}'
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.comment_issue("DEAP-10", "Dev complete, tests pass. Ready for review.")
        self.assertTrue(success)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.method, "POST")
        self.assertTrue(req.full_url.endswith("/rest/api/2/issue/DEAP-10/comment"))
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["body"], "Dev complete, tests pass. Ready for review.")

    @patch("urllib.request.urlopen")
    def test_dynamic_workflow_transition_resolution(self, mock_urlopen):
        """Verify dynamic matching of workflow transitions by name/target status."""
        transitions_data = json.dumps({
            "transitions": [
                {"id": "11", "name": "Start Progress", "to": {"name": "In Progress"}},
                {"id": "21", "name": "Resolve Issue", "to": {"name": "Resolved"}},
                {"id": "31", "name": "Done", "to": {"name": "Done"}},
                {"id": "41", "name": "Ready for Review", "to": {"name": "Under Review"}},
            ]
        }).encode("utf-8")

        resp_get = MagicMock()
        resp_get.status = 200
        resp_get.headers = {}
        resp_get.read.return_value = transitions_data
        resp_get.__enter__.return_value = resp_get

        resp_post = MagicMock()
        resp_post.status = 204
        resp_post.headers = {}
        resp_post.read.return_value = b""
        resp_post.__enter__.return_value = resp_post

        # Test transition resolution for 'Resolved'
        mock_urlopen.side_effect = [resp_get, resp_post]
        success = self.provider.transition_issue("DEAP-10", "Resolved", resolution_note="Fixed in build 42")
        self.assertTrue(success)

        post_req = mock_urlopen.call_args_list[1][0][0]
        self.assertEqual(post_req.method, "POST")
        self.assertTrue(post_req.full_url.endswith("/rest/api/2/issue/DEAP-10/transitions"))
        post_payload = json.loads(post_req.data.decode("utf-8"))
        self.assertEqual(post_payload["transition"]["id"], "21")
        self.assertEqual(post_payload["update"]["comment"][0]["add"]["body"], "Fixed in build 42")

        # Test transition resolution for 'status::ready-for-review'
        resp_get2 = MagicMock()
        resp_get2.status = 200
        resp_get2.headers = {}
        resp_get2.read.return_value = transitions_data
        resp_get2.__enter__.return_value = resp_get2

        resp_post2 = MagicMock()
        resp_post2.status = 204
        resp_post2.headers = {}
        resp_post2.read.return_value = b""
        resp_post2.__enter__.return_value = resp_post2

        mock_urlopen.side_effect = [resp_get2, resp_post2]
        success2 = self.provider.transition_issue("DEAP-10", "status::ready-for-review")
        self.assertTrue(success2)
        post_payload2 = json.loads(mock_urlopen.call_args_list[3][0][0].data.decode("utf-8"))
        self.assertEqual(post_payload2["transition"]["id"], "41")

    @patch("urllib.request.urlopen")
    def test_create_issue_link_success(self, mock_urlopen):
        """Verify create_issue_link sends POST /rest/api/2/issueLink with link type."""
        resp = MagicMock()
        resp.status = 201
        resp.headers = {}
        resp.read.return_value = b""
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.create_issue_link("DEAP-1", "DEAP-2", link_type="Blocks")
        self.assertTrue(success)

        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.method, "POST")
        self.assertTrue(req.full_url.endswith("/rest/api/2/issueLink"))
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["type"]["name"], "Blocks")
        self.assertEqual(payload["inwardIssue"]["key"], "DEAP-1")
        self.assertEqual(payload["outwardIssue"]["key"], "DEAP-2")

    @patch("time.sleep")
    @patch("urllib.request.urlopen")
    def test_http_429_rate_limit_backoff(self, mock_urlopen, mock_sleep):
        """Verify automatic backoff and retry on HTTP 429 using Retry-After header."""
        err_headers = {"Retry-After": "2"}
        http_429_err = urllib.error.HTTPError(
            url="https://jira.example.com/rest/api/2/search",
            code=429,
            msg="Rate Limited",
            hdrs=err_headers,
            fp=None,
        )

        resp_success = MagicMock()
        resp_success.status = 200
        resp_success.headers = {}
        resp_success.read.return_value = b'{"status": "recovered"}'
        resp_success.__enter__.return_value = resp_success

        mock_urlopen.side_effect = [http_429_err, resp_success]

        status_code, data, _ = self.provider._api_request("rest/api/2/search")
        self.assertEqual(status_code, 200)
        self.assertEqual(data.get("status"), "recovered")
        mock_sleep.assert_called_once_with(2.0)

    def test_offline_mode(self):
        """Verify offline mode safely returns empty lists / None without making network requests."""
        offline_provider = JiraV2V3Provider(offline=True)
        self.assertEqual(offline_provider.list_issues(), [])
        self.assertIsNone(offline_provider.get_issue("DEAP-1"))
        self.assertIsNone(offline_provider.create_issue("Title", "Body"))
        self.assertFalse(offline_provider.edit_issue("DEAP-1", title="Title"))
        self.assertFalse(offline_provider.add_label("DEAP-1", "label"))
        self.assertFalse(offline_provider.comment_issue("DEAP-1", "comment"))
        self.assertFalse(offline_provider.transition_issue("DEAP-1", "Done"))
        self.assertFalse(offline_provider.create_issue_link("DEAP-1", "DEAP-2"))


class TestJiraTrackerDetectionAndRules(unittest.TestCase):
    def test_tracker_provider_detection_jira(self):
        """Verify Jira tracker provider detection across CLI, env vars, and rules."""
        # 1. CLI override
        self.assertEqual(detect_tracker_provider(cli_provider="jira"), "jira")

        # 2. Env vars
        with patch.dict(os.environ, {"JIRA_SERVER_URL": "https://jira.corp.internal"}, clear=True):
            self.assertEqual(detect_tracker_provider(), "jira")

        with patch.dict(os.environ, {"JIRA_PROJECT_KEY": "SAFETY"}, clear=True):
            self.assertEqual(detect_tracker_provider(), "jira")

        # 3. Codebase rules
        rules = {"tracker_rules": {"provider": "jira"}}
        self.assertEqual(detect_tracker_provider(rules=rules), "jira")

    def test_create_tracker_provider_jira(self):
        rules = {
            "tracker_rules": {
                "server_url": "https://jira.custom.corp",
                "project_key": "DEAP",
            }
        }
        provider = create_tracker_provider("jira", rules=rules, offline=False)
        self.assertIsInstance(provider, JiraV2V3Provider)
        self.assertEqual(provider.server_url, "https://jira.custom.corp")
        self.assertEqual(provider.project_key, "DEAP")

    def test_load_codebase_rules_with_jira_defaults(self):
        rules = load_codebase_rules(os.getcwd(), provider="jira")
        self.assertEqual(rules["tracker_rules"]["provider"], "jira")
        self.assertEqual(rules["tracker_rules"]["labels"]["epic"], "type::epic")
        self.assertEqual(rules["tracker_rules"]["labels"]["feature"], "type::feature")
        self.assertEqual(rules["tracker_rules"]["labels"]["user_story"], "type::user-story")
        self.assertEqual(rules["tracker_rules"]["labels"]["use_case"], "type::use-case")
        self.assertEqual(rules["tracker_rules"]["labels"]["resolved"], "status::fixed-resolved")

    def test_jira_scoped_labels(self):
        rules = {"tracker_rules": {"provider": "jira"}}
        self.assertEqual(get_structural_label("Epic", rules), "type::epic")
        self.assertEqual(get_structural_label("Feature", rules), "type::feature")
        self.assertEqual(get_structural_label("User Story", rules), "type::user-story")
        self.assertEqual(get_structural_label("Use Case", rules), "type::use-case")
        self.assertEqual(get_resolved_label(rules), "status::fixed-resolved")

    def test_jira_alias_compatibility(self):
        """Verify JiraRESTProvider is an alias to JiraV2V3Provider for backwards compatibility."""
        self.assertIs(JiraRESTProvider, JiraV2V3Provider)


if __name__ == "__main__":
    unittest.main()
