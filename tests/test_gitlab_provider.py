import os
import sys
import json
import unittest
import netrc
from unittest.mock import patch, MagicMock

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    GitLabV4Provider,
    GitHubCLIProvider,
    parse_git_remote_url,
    detect_tracker_provider,
    load_codebase_rules,
    get_structural_label,
    get_resolved_label,
    create_tracker_provider,
    DEFAULT_GITLAB_TRACKER_RULES,
    DEFAULT_GITLAB_STRUCTURAL_LABELS,
)


class TestGitLabRemoteParser(unittest.TestCase):
    def test_parse_https_url(self):
        url = "https://gitlab.com/gintatkinson/DEAP-spec-core.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "gintatkinson/DEAP-spec-core")
        self.assertEqual(info["server_url"], "https://gitlab.com")
        self.assertEqual(info["host"], "gitlab.com")

    def test_parse_custom_gitlab_domain(self):
        url = "https://gitlab.internal.corp/safety-team/uas/uas-core.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "safety-team/uas/uas-core")
        self.assertEqual(info["server_url"], "https://gitlab.internal.corp")
        self.assertEqual(info["host"], "gitlab.internal.corp")

    def test_parse_ssh_scp_style(self):
        url = "git@gitlab.com:gintatkinson/DEAP-spec-core.git"
        info = parse_git_remote_url(url)
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "gintatkinson/DEAP-spec-core")
        self.assertEqual(info["server_url"], "https://gitlab.com")

    def test_parse_github_url(self):
        url = "https://github.com/gintatkinson/DEAP-spec-core.git"
        info = parse_git_remote_url(url)
        self.assertFalse(info["is_gitlab"])
        self.assertEqual(info["project_path"], "gintatkinson/DEAP-spec-core")
        self.assertEqual(info["server_url"], "https://github.com")


class TestGitLabProviderResolution(unittest.TestCase):
    @patch.dict(os.environ, {
        "GITLAB_URL": "https://gitlab.example.com",
        "CI_PROJECT_PATH": "my-org/my-project",
        "GITLAB_TOKEN": "glpat-testtoken123"
    }, clear=True)
    def test_env_resolution(self):
        provider = GitLabV4Provider()
        self.assertEqual(provider.server_url, "https://gitlab.example.com")
        self.assertEqual(provider.raw_project_id, "my-org/my-project")
        self.assertEqual(provider.project_id_encoded, "my-org%2Fmy-project")
        self.assertEqual(provider.token, "glpat-testtoken123")
        self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch.dict(os.environ, {
        "CI_SERVER_URL": "https://gitlab.ci.corp",
        "CI_PROJECT_PATH": "group/subgroup/project",
        "CI_JOB_TOKEN": "job-token-xyz"
    }, clear=True)
    def test_ci_job_token_resolution(self):
        provider = GitLabV4Provider()
        self.assertEqual(provider.server_url, "https://gitlab.ci.corp")
        self.assertEqual(provider.project_id_encoded, "group%2Fsubgroup%2Fproject")
        self.assertEqual(provider.token, "job-token-xyz")
        self.assertEqual(provider.token_type, "JOB-TOKEN")

    def test_numeric_project_id(self):
        provider = GitLabV4Provider(
            server_url="https://gitlab.com",
            project_id="123456",
            token="test-token"
        )
        self.assertEqual(provider.project_id_encoded, "123456")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc")
    def test_netrc_resolution(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            mock_netrc_inst = MagicMock()
            mock_netrc_inst.authenticators.return_value = ("user", "account", "netrc-glpat-token")
            mock_netrc_class.return_value = mock_netrc_inst

            provider = GitLabV4Provider(project_id="my-org/my-project")
            self.assertEqual(provider.token, "netrc-glpat-token")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")
            mock_netrc_inst.authenticators.assert_called_once_with("gitlab.com")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc")
    def test_netrc_resolution_custom_host(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            mock_netrc_inst = MagicMock()
            mock_netrc_inst.authenticators.return_value = ("user", "account", "custom-corp-token")
            mock_netrc_class.return_value = mock_netrc_inst

            provider = GitLabV4Provider(
                server_url="https://gitlab.internal.defense.gov",
                project_id="uas-safety",
            )
            self.assertEqual(provider.token, "custom-corp-token")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")
            mock_netrc_inst.authenticators.assert_called_once_with("gitlab.internal.defense.gov")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc", side_effect=FileNotFoundError("~/.netrc missing"))
    def test_netrc_resolution_missing_file_fallback(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4Provider(project_id="uas-safety")
            self.assertIsNone(provider.token)
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")


class TestGitLabApiOperations(unittest.TestCase):
    def setUp(self):
        self.provider = GitLabV4Provider(
            server_url="https://gitlab.com",
            project_id="owner/repo",
            token="glpat-mock-token"
        )

    @patch("urllib.request.urlopen")
    def test_list_issues_pagination(self, mock_urlopen):
        page1_data = json.dumps([
            {"iid": 1, "title": "Epic 1", "state": "opened", "labels": ["type::epic"]},
            {"iid": 2, "title": "Feature 1", "state": "opened", "labels": ["type::feature"]}
        ]).encode("utf-8")
        
        page2_data = json.dumps([
            {"iid": 3, "title": "User Story 1", "state": "closed", "labels": ["type::user-story"]}
        ]).encode("utf-8")

        resp1 = MagicMock()
        resp1.status = 200
        resp1.headers = {"X-Next-Page": "2"}
        resp1.read.return_value = page1_data
        resp1.__enter__.return_value = resp1

        resp2 = MagicMock()
        resp2.status = 200
        resp2.headers = {"X-Next-Page": ""}
        resp2.read.return_value = page2_data
        resp2.__enter__.return_value = resp2

        mock_urlopen.side_effect = [resp1, resp2]

        issues = self.provider.list_issues()
        self.assertEqual(len(issues), 3)
        self.assertEqual(issues[0]["number"], 1)
        self.assertEqual(issues[0]["state"], "OPENED")
        self.assertEqual(issues[2]["number"], 3)
        self.assertEqual(issues[2]["state"], "CLOSED")

    @patch("urllib.request.urlopen")
    def test_create_issue(self, mock_urlopen):
        resp_data = json.dumps({"iid": 42, "title": "New Issue", "state": "opened"}).encode("utf-8")
        resp = MagicMock()
        resp.status = 201
        resp.headers = {}
        resp.read.return_value = resp_data
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        created = self.provider.create_issue("New Issue", "Description", labels=["type::feature"])
        self.assertIsNotNone(created)
        self.assertEqual(created["number"], 42)

    @patch("urllib.request.urlopen")
    def test_edit_issue(self, mock_urlopen):
        resp = MagicMock()
        resp.status = 200
        resp.headers = {}
        resp.read.return_value = b'{"iid": 42}'
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.edit_issue(42, "Updated description")
        self.assertTrue(success)

    @patch("urllib.request.urlopen")
    def test_edit_issue_title(self, mock_urlopen):
        resp = MagicMock()
        resp.status = 200
        resp.headers = {}
        resp.read.return_value = b'{"iid": 42, "title": "Updated Title"}'
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.edit_issue_title(42, "Updated Title")
        self.assertTrue(success)

    @patch("urllib.request.urlopen")
    def test_add_label(self, mock_urlopen):
        resp = MagicMock()
        resp.status = 200
        resp.headers = {}
        resp.read.return_value = b'{"iid": 42}'
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.add_label(42, "status::ready-for-review")
        self.assertTrue(success)

    @patch("urllib.request.urlopen")
    def test_comment_issue(self, mock_urlopen):
        resp = MagicMock()
        resp.status = 201
        resp.headers = {}
        resp.read.return_value = b'{"id": 101, "body": "Test comment"}'
        resp.__enter__.return_value = resp
        mock_urlopen.return_value = resp

        success = self.provider.comment_issue(42, "Test comment")
        self.assertTrue(success)

    def test_gitlab_provider_offline_mode(self):
        offline_provider = GitLabV4Provider(offline=True)
        self.assertEqual(offline_provider.list_issues(), [])

    @patch("urllib.request.urlopen")
    def test_api_error_handling(self, mock_urlopen):
        mock_urlopen.side_effect = Exception("API connection error")
        created = self.provider.create_issue("Title", "Body")
        self.assertIsNone(created)
        success = self.provider.edit_issue(42, "Body")
        self.assertFalse(success)


class TestTrackerProviderDetection(unittest.TestCase):
    def test_cli_provider_override(self):
        prov = detect_tracker_provider(cli_provider="gitlab")
        self.assertEqual(prov, "gitlab")

    @patch.dict(os.environ, {"GITLAB_CI": "true"}, clear=True)
    def test_gitlab_ci_detection(self):
        prov = detect_tracker_provider()
        self.assertEqual(prov, "gitlab")

    @patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}, clear=True)
    def test_github_actions_detection(self):
        prov = detect_tracker_provider()
        self.assertEqual(prov, "github")

    def test_rules_loading_with_gitlab_defaults(self):
        rules = load_codebase_rules(os.getcwd(), provider="gitlab")
        self.assertEqual(rules["tracker_rules"]["provider"], "gitlab")
        self.assertEqual(rules["tracker_rules"]["labels"]["epic"], "type::epic")
        self.assertEqual(rules["tracker_rules"]["labels"]["feature"], "type::feature")
        self.assertEqual(rules["tracker_rules"]["labels"]["user_story"], "type::user-story")
        self.assertEqual(rules["tracker_rules"]["labels"]["use_case"], "type::use-case")
        self.assertEqual(rules["tracker_rules"]["labels"]["resolved"], "status::fixed-resolved")

    def test_rules_loading_gitlab_explicit_override_cleans_github_keys_and_labels(self):
        with patch("reconcile_backlog.resolve_codebase_rules_path", return_value=None):
            rules = load_codebase_rules(os.getcwd(), provider="gitlab")
            self.assertEqual(rules["tracker_rules"]["keys"], DEFAULT_GITLAB_TRACKER_RULES["keys"])
            self.assertEqual(rules["tracker_rules"]["labels"], DEFAULT_GITLAB_TRACKER_RULES["labels"])
            self.assertEqual(rules["tracker_rules"]["keys"]["issue_id"], "iid")
            self.assertEqual(rules["tracker_rules"]["keys"]["open_state_value"], "OPENED")

    def test_rules_loading_gitlab_provider_in_loaded_rules_merges_custom_rules(self):
        custom_config = {
            "tracker_rules": {
                "provider": "gitlab",
                "labels": {"custom_label": "type::custom"},
                "keys": {"custom_key": "custom_val"}
            }
        }
        with patch("reconcile_backlog.resolve_codebase_rules_path", return_value="/fake/codebase_rules.json"):
            with patch("builtins.open", MagicMock()):
                with patch("json.load", return_value=custom_config):
                    rules = load_codebase_rules(os.getcwd(), provider="gitlab")
                    self.assertEqual(rules["tracker_rules"]["provider"], "gitlab")
                    self.assertEqual(rules["tracker_rules"]["labels"]["custom_label"], "type::custom")
                    self.assertEqual(rules["tracker_rules"]["labels"]["epic"], "type::epic")
                    self.assertEqual(rules["tracker_rules"]["keys"]["custom_key"], "custom_val")

    def test_rules_loading_github_in_loaded_rules_overridden_by_gitlab_provider(self):
        github_config = {
            "tracker_rules": {
                "provider": "github",
                "labels": {"epic": "gh-epic", "stale_gh_label": "gh-val"},
                "keys": {"issue_id": "number", "open_state_value": "OPEN"}
            }
        }
        with patch("reconcile_backlog.resolve_codebase_rules_path", return_value="/fake/codebase_rules.json"):
            with patch("builtins.open", MagicMock()):
                with patch("json.load", return_value=github_config):
                    rules = load_codebase_rules(os.getcwd(), provider="gitlab")
                    self.assertEqual(rules["tracker_rules"]["provider"], "gitlab")
                    self.assertEqual(rules["tracker_rules"]["labels"], DEFAULT_GITLAB_TRACKER_RULES["labels"])
                    self.assertEqual(rules["tracker_rules"]["keys"], DEFAULT_GITLAB_TRACKER_RULES["keys"])
                    self.assertEqual(rules["tracker_rules"]["keys"]["issue_id"], "iid")
                    self.assertNotIn("stale_gh_label", rules["tracker_rules"]["labels"])

    def test_gitlab_scoped_labels(self):
        rules = {"tracker_rules": {"provider": "gitlab"}}
        self.assertEqual(get_structural_label("Epic", rules), "type::epic")
        self.assertEqual(get_structural_label("Feature", rules), "type::feature")
        self.assertEqual(get_structural_label("User Story", rules), "type::user-story")
        self.assertEqual(get_structural_label("Use Case", rules), "type::use-case")
        self.assertEqual(get_resolved_label(rules), "status::fixed-resolved")

    def test_create_tracker_provider_gitlab(self):
        rules = {"tracker_rules": {"server_url": "https://gitlab.example.com", "project_id": "group/project"}}
        provider = create_tracker_provider("gitlab", rules=rules, offline=False)
        self.assertIsInstance(provider, GitLabV4Provider)
        self.assertEqual(provider.server_url, "https://gitlab.example.com")
        self.assertEqual(provider.raw_project_id, "group/project")

    def test_create_tracker_provider_github(self):
        rules = {"tracker_rules": {}}
        provider = create_tracker_provider("github", rules=rules, offline=True)
        self.assertIsInstance(provider, GitHubCLIProvider)
        self.assertTrue(provider.offline)

    def test_gitlab_close_comments(self):
        close_comments = DEFAULT_GITLAB_TRACKER_RULES.get("close_comments", {})
        self.assertIn("epic", close_comments)
        self.assertIn("user_story", close_comments)
        self.assertIn("use_case", close_comments)


class TestMultiProviderBacklogLinkSynthesis(unittest.TestCase):
    def setUp(self):
        import tempfile
        self.temp_dir = tempfile.TemporaryDirectory()
        self.epic_path = os.path.join(self.temp_dir.name, "epic-01.md")
        epic_content = (
            "# Epic 01: Core Platform\n\n"
            "## 2. Requirements & Checklist\n\n"
            "### Associated Features\n"
            "- [ ] #0 - Feature One\n"
        )
        with open(self.epic_path, "w", encoding="utf-8") as f:
            f.write(epic_content)

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_gitlab_blob_url_synthesis(self):
        """Assert GitLab blob URL uses /-/blob/ syntax when provider is gitlab."""
        from reconcile_backlog import reconcile_epic_checklists
        rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP-spec-core"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.com",
            }
        }
        child_features = [("feat-01-auth", "Feature One")]
        child_stories = []
        child_usecases = []
        epic_titles = {}
        feature_titles = {"Feature One": 101}
        story_titles = {}
        usecase_titles = {}

        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            reconcile_epic_checklists(
                self.epic_path,
                child_features,
                child_stories,
                child_usecases,
                epic_titles,
                feature_titles,
                story_titles,
                usecase_titles,
                rules
            )

        with open(self.epic_path, "r", encoding="utf-8") as f:
            updated_content = f.read()

        # Must synthesize GitLab blob URL format with /-/blob/
        expected_url = "https://gitlab.com/gintatkinson/DEAP-spec-core/-/blob/main/docs/features/feat-01-auth.md"
        self.assertIn(expected_url, updated_content)
        self.assertNotIn("/blob/main/docs/features/feat-01-auth.md", updated_content.replace("/-/blob/", "/REPLACED/"))

    def test_gitlab_blob_url_synthesis_custom_host(self):
        """Assert GitLab blob URL with custom host uses /-/blob/ syntax."""
        from reconcile_backlog import reconcile_epic_checklists
        rules = {
            "meta": {"upstream_repository": "internal-group/sub-project"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.internal.corp",
            }
        }
        child_features = [("feat-02-core", "Feature Two")]
        child_stories = []
        child_usecases = []
        epic_titles = {}
        feature_titles = {"Feature Two": 102}
        story_titles = {}
        usecase_titles = {}

        with patch.dict(os.environ, {"GITLAB_URL": "https://gitlab.internal.corp"}):
            with patch("reconcile_backlog.get_current_branch", return_value="develop"):
                reconcile_epic_checklists(
                    self.epic_path,
                    child_features,
                    child_stories,
                    child_usecases,
                    epic_titles,
                    feature_titles,
                    story_titles,
                    usecase_titles,
                    rules
                )

        with open(self.epic_path, "r", encoding="utf-8") as f:
            updated_content = f.read()

        expected_url = "https://gitlab.internal.corp/internal-group/sub-project/-/blob/develop/docs/features/feat-02-core.md"
        self.assertIn(expected_url, updated_content)

    def test_github_blob_url_synthesis(self):
        """Assert GitHub blob URL uses /blob/ syntax when provider is github."""
        from reconcile_backlog import reconcile_epic_checklists
        rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP-spec-core"},
            "tracker_rules": {
                "provider": "github",
            }
        }
        child_features = [("feat-01-auth", "Feature One")]
        child_stories = []
        child_usecases = []
        epic_titles = {}
        feature_titles = {"Feature One": 101}
        story_titles = {}
        usecase_titles = {}

        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            reconcile_epic_checklists(
                self.epic_path,
                child_features,
                child_stories,
                child_usecases,
                epic_titles,
                feature_titles,
                story_titles,
                usecase_titles,
                rules
            )

        with open(self.epic_path, "r", encoding="utf-8") as f:
            updated_content = f.read()

        expected_url = "https://github.com/gintatkinson/DEAP-spec-core/blob/main/docs/features/feat-01-auth.md"
        self.assertIn(expected_url, updated_content)

    def test_sanitize_source_references_gitlab(self):
        """Assert sanitize_source_references produces GitLab blob URLs when provider is gitlab."""
        from reconcile_backlog import sanitize_source_references
        rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP-spec-core"},
            "tracker_rules": {"provider": "gitlab"}
        }
        raw_text = "See file:///workspace/DEAP-spec-core/docs/features/feat-01-auth.md for details."
        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            sanitized = sanitize_source_references(raw_text, workspace_dir="/workspace/DEAP-spec-core", rules=rules)
        self.assertIn("https://gitlab.com/gintatkinson/DEAP-spec-core/-/blob/main/docs/features/feat-01-auth.md", sanitized)

    def test_get_blob_url_base_jira_with_gitlab_remote(self):
        """Assert Jira tracker with GitLab remote produces GitLab blob URL base."""
        from reconcile_backlog import get_blob_url_base
        rules = {
            "meta": {"upstream_repository": "safety-team/uas-core"},
            "tracker_rules": {"provider": "jira"}
        }
        mock_remote = {
            "is_gitlab": True,
            "server_url": "https://gitlab.internal.corp",
            "project_path": "safety-team/uas-core",
            "host": "gitlab.internal.corp"
        }
        with patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote):
            base_url = get_blob_url_base(rules=rules, workspace_dir="/tmp/test", branch="release-1.0")
        self.assertEqual(base_url, "https://gitlab.internal.corp/safety-team/uas-core/-/blob/release-1.0")


if __name__ == "__main__":
    unittest.main()
