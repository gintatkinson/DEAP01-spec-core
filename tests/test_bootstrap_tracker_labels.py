import json
import netrc
import os
import sys
import tempfile
import unittest
import urllib.error
from unittest.mock import MagicMock, patch

# Add skills/spec-orchestrator/scripts to sys.path
scripts_dir = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "skills", "spec-orchestrator", "scripts")
)
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from bootstrap_tracker_labels import (
    DEFAULT_GITHUB_LABELS,
    DEFAULT_GITLAB_LABELS,
    LABEL_PRESENTATION,
    GitHubCLILabelProvider,
    GitLabV4LabelProvider,
    bootstrap_labels,
    detect_tracker_provider,
    find_workspace_dir,
    get_label_presentation,
    load_labels,
    main,
    normalize_color,
    parse_git_remote_url,
    resolve_codebase_rules_path,
)


class TestColorNormalizationAndPresentation(unittest.TestCase):
    def test_normalize_color_with_hash(self):
        self.assertEqual(normalize_color("800080", with_hash=True), "#800080")
        self.assertEqual(normalize_color("#800080", with_hash=True), "#800080")
        self.assertEqual(normalize_color("0e8a16", with_hash=True), "#0E8A16")
        self.assertEqual(normalize_color("#0E8A16", with_hash=True), "#0E8A16")
        self.assertEqual(normalize_color(None, with_hash=True), "#0E8A16")
        self.assertEqual(normalize_color("", with_hash=True), "#0E8A16")
        self.assertEqual(normalize_color("invalid_long_string", with_hash=True), "#0E8A16")

    def test_normalize_color_without_hash(self):
        self.assertEqual(normalize_color("800080", with_hash=False), "800080")
        self.assertEqual(normalize_color("#800080", with_hash=False), "800080")
        self.assertEqual(normalize_color("#0E8A16", with_hash=False), "0e8a16")
        self.assertEqual(normalize_color(None, with_hash=False), "0e8a16")

    def test_presentation_for_unscoped_keys(self):
        color, desc = get_label_presentation("epic", "epic")
        self.assertEqual(color, "800080")
        self.assertIn("Epic", desc)

        color, desc = get_label_presentation("feature", "feature")
        self.assertEqual(color, "0366d6")

        color, desc = get_label_presentation("user_story", "user-story")
        self.assertEqual(color, "0e8a16")

        color, desc = get_label_presentation("use_case", "use-case")
        self.assertEqual(color, "fbca04")

        color, desc = get_label_presentation("resolved", "status:fixed-resolved")
        self.assertEqual(color, "0e8a16")

    def test_presentation_for_gitlab_scoped_labels(self):
        color, desc = get_label_presentation("epic", "type::epic")
        self.assertEqual(color, "800080")

        color, desc = get_label_presentation("feature", "type::feature")
        self.assertEqual(color, "0366d6")

        color, desc = get_label_presentation("user_story", "type::user-story")
        self.assertEqual(color, "0e8a16")

        color, desc = get_label_presentation("use_case", "type::use-case")
        self.assertEqual(color, "fbca04")

        color, desc = get_label_presentation("ready_for_review", "status::ready-for-review")
        self.assertEqual(color, "6f42c1")

        color, desc = get_label_presentation("resolved", "status::fixed-resolved")
        self.assertEqual(color, "0e8a16")

    def test_presentation_fallback(self):
        color, desc = get_label_presentation("custom_key", "custom_label_name")
        self.assertEqual(color, "ededed")
        self.assertEqual(desc, "custom_label_name")


class TestGitRemoteParser(unittest.TestCase):
    def test_parse_https_gitlab(self):
        info = parse_git_remote_url("https://gitlab.com/owner/project.git")
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "owner/project")
        self.assertEqual(info["server_url"], "https://gitlab.com")

    def test_parse_ssh_gitlab(self):
        info = parse_git_remote_url("git@gitlab.com:group/subgroup/project.git")
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "group/subgroup/project")
        self.assertEqual(info["server_url"], "https://gitlab.com")

    def test_parse_custom_gitlab_domain(self):
        info = parse_git_remote_url("https://gitlab.internal.defense.gov/uas/safety.git")
        self.assertTrue(info["is_gitlab"])
        self.assertEqual(info["project_path"], "uas/safety")
        self.assertEqual(info["server_url"], "https://gitlab.internal.defense.gov")

    def test_parse_github_url(self):
        info = parse_git_remote_url("https://github.com/owner/repo.git")
        self.assertFalse(info["is_gitlab"])
        self.assertEqual(info["project_path"], "owner/repo")
        self.assertEqual(info["server_url"], "https://github.com")

    def test_parse_empty_url(self):
        info = parse_git_remote_url("")
        self.assertFalse(info["is_gitlab"])
        self.assertIsNone(info["project_path"])


class TestProviderDetection(unittest.TestCase):
    def test_cli_override(self):
        self.assertEqual(detect_tracker_provider(cli_provider="gitlab"), "gitlab")
        self.assertEqual(detect_tracker_provider(cli_provider="github"), "github")

    @patch.dict(os.environ, {"TRACKER_PROVIDER": "gitlab"}, clear=True)
    def test_env_tracker_provider(self):
        self.assertEqual(detect_tracker_provider(), "gitlab")

    @patch.dict(os.environ, {"GITLAB_CI": "true"}, clear=True)
    def test_env_gitlab_ci(self):
        self.assertEqual(detect_tracker_provider(), "gitlab")

    @patch.dict(os.environ, {"CI_SERVER_URL": "https://gitlab.example.com"}, clear=True)
    def test_env_ci_server_url(self):
        self.assertEqual(detect_tracker_provider(), "gitlab")

    @patch.dict(os.environ, {"GITLAB_TOKEN": "glpat-secret"}, clear=True)
    def test_env_gitlab_token(self):
        self.assertEqual(detect_tracker_provider(), "gitlab")

    @patch.dict(os.environ, {"CI_JOB_TOKEN": "job-token-xyz"}, clear=True)
    def test_env_ci_job_token(self):
        self.assertEqual(detect_tracker_provider(), "gitlab")

    @patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}, clear=True)
    def test_env_github_actions(self):
        self.assertEqual(detect_tracker_provider(), "github")

    def test_rules_provider_detection(self):
        rules = {"tracker_rules": {"provider": "gitlab"}}
        self.assertEqual(detect_tracker_provider(rules=rules), "gitlab")

    @patch("bootstrap_tracker_labels.get_git_remote_info")
    def test_git_remote_gitlab_detection(self, mock_remote):
        mock_remote.return_value = {"is_gitlab": True}
        self.assertEqual(detect_tracker_provider(workspace_dir="/some/dir"), "gitlab")

    def test_default_fallback(self):
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(detect_tracker_provider(), "github")


class TestRulesLoading(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()

    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_resolve_candidate_paths(self):
        # 1. CODEBASE_RULES_PATH
        custom_path = os.path.join(self.temp_dir, "custom_rules.json")
        with open(custom_path, "w", encoding="utf-8") as f:
            json.dump({"meta": {}}, f)

        with patch.dict(os.environ, {"CODEBASE_RULES_PATH": custom_path}):
            resolved = resolve_codebase_rules_path(self.temp_dir)
            self.assertEqual(resolved, custom_path)

        # 2. .pipeline/logical-ui/codebase_rules.json
        pipeline_ui = os.path.join(self.temp_dir, ".pipeline", "logical-ui")
        os.makedirs(pipeline_ui, exist_ok=True)
        ui_path = os.path.join(pipeline_ui, "codebase_rules.json")
        with open(ui_path, "w", encoding="utf-8") as f:
            json.dump({"meta": {}}, f)

        with patch.dict(os.environ, {}, clear=True):
            resolved = resolve_codebase_rules_path(self.temp_dir)
            self.assertEqual(resolved, ui_path)

        # 3. .pipeline/codebase_rules.json
        os.remove(ui_path)
        p_path = os.path.join(self.temp_dir, ".pipeline", "codebase_rules.json")
        with open(p_path, "w", encoding="utf-8") as f:
            json.dump({"meta": {}}, f)

        with patch.dict(os.environ, {}, clear=True):
            resolved = resolve_codebase_rules_path(self.temp_dir)
            self.assertEqual(resolved, p_path)

        # 4. codebase_rules.json
        os.remove(p_path)
        root_path = os.path.join(self.temp_dir, "codebase_rules.json")
        with open(root_path, "w", encoding="utf-8") as f:
            json.dump({"meta": {}}, f)

        with patch.dict(os.environ, {}, clear=True):
            resolved = resolve_codebase_rules_path(self.temp_dir)
            self.assertEqual(resolved, root_path)

    def test_load_labels_github_default(self):
        labels = load_labels(self.temp_dir, provider="github")
        self.assertEqual(labels["epic"], "epic")
        self.assertEqual(labels["feature"], "feature")
        self.assertEqual(labels["resolved"], "status:fixed-resolved")

    def test_load_labels_gitlab_default(self):
        labels = load_labels(self.temp_dir, provider="gitlab")
        self.assertEqual(labels["epic"], "type::epic")
        self.assertEqual(labels["feature"], "type::feature")
        self.assertEqual(labels["user_story"], "type::user-story")
        self.assertEqual(labels["use_case"], "type::use-case")
        self.assertEqual(labels["resolved"], "status::fixed-resolved")

    def test_load_labels_from_file_with_custom(self):
        root_path = os.path.join(self.temp_dir, "codebase_rules.json")
        custom_data = {
            "tracker_rules": {
                "provider": "gitlab",
                "labels": {
                    "epic": "type::epic",
                    "custom_key": "type::custom",
                },
            }
        }
        with open(root_path, "w", encoding="utf-8") as f:
            json.dump(custom_data, f)

        labels = load_labels(self.temp_dir, provider="gitlab")
        self.assertEqual(labels["custom_key"], "type::custom")
        self.assertEqual(labels["epic"], "type::epic")


class TestGitLabV4LabelProvider(unittest.TestCase):
    def setUp(self):
        self.provider = GitLabV4LabelProvider(
            server_url="https://gitlab.example.com",
            project_id="uas-team/safety-core",
            token="glpat-test-token",
        )

    def test_project_id_encoding(self):
        self.assertEqual(self.provider.project_id_encoded, "uas-team%2Fsafety-core")

        numeric_provider = GitLabV4LabelProvider(
            server_url="https://gitlab.example.com",
            project_id="12345",
            token="glpat-token",
        )
        self.assertEqual(numeric_provider.project_id_encoded, "12345")

    def test_job_token_header_resolution(self):
        with patch.dict(os.environ, {"CI_JOB_TOKEN": "job-token-123"}, clear=True):
            prov = GitLabV4LabelProvider(project_id="123")
            self.assertEqual(prov.token, "job-token-123")
            self.assertEqual(prov.token_type, "JOB-TOKEN")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc")
    def test_netrc_token_resolution(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            mock_netrc_inst = MagicMock()
            mock_netrc_inst.authenticators.return_value = ("user", "account", "netrc-secret-token")
            mock_netrc_class.return_value = mock_netrc_inst

            prov = GitLabV4LabelProvider(project_id="123")
            self.assertEqual(prov.token, "netrc-secret-token")
            self.assertEqual(prov.token_type, "PRIVATE-TOKEN")
            mock_netrc_inst.authenticators.assert_called_once_with("gitlab.com")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc")
    def test_netrc_token_resolution_custom_host(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            mock_netrc_inst = MagicMock()
            mock_netrc_inst.authenticators.return_value = ("user", "account", "custom-netrc-token")
            mock_netrc_class.return_value = mock_netrc_inst

            prov = GitLabV4LabelProvider(
                server_url="https://gitlab.internal.defense.gov",
                project_id="uas/safety",
            )
            self.assertEqual(prov.token, "custom-netrc-token")
            self.assertEqual(prov.token_type, "PRIVATE-TOKEN")
            mock_netrc_inst.authenticators.assert_called_once_with("gitlab.internal.defense.gov")

    @patch("shutil.which", return_value=None)
    @patch("netrc.netrc", side_effect=FileNotFoundError("~/.netrc not found"))
    def test_netrc_token_resolution_missing_file(self, mock_netrc_class, mock_which):
        with patch.dict(os.environ, {}, clear=True):
            prov = GitLabV4LabelProvider(project_id="123")
            self.assertIsNone(prov.token)
            self.assertEqual(prov.token_type, "PRIVATE-TOKEN")

    @patch("urllib.request.urlopen")
    def test_create_label_success(self, mock_urlopen):
        mock_resp = MagicMock()
        mock_resp.status = 201
        mock_resp.__enter__.return_value = mock_resp
        mock_urlopen.return_value = mock_resp

        success = self.provider.create_label("type::epic", "Epic description", "800080")
        self.assertTrue(success)
        self.assertTrue(mock_urlopen.called)
        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_method(), "POST")
        self.assertIn("projects/uas-team%2Fsafety-core/labels", req.full_url)
        self.assertEqual(req.headers.get("Private-token"), "glpat-test-token")
        payload = json.loads(req.data.decode("utf-8"))
        self.assertEqual(payload["name"], "type::epic")
        self.assertEqual(payload["color"], "#800080")

    @patch("urllib.request.urlopen")
    def test_create_label_idempotent_409_conflict(self, mock_urlopen):
        # Simulate HTTP 409 Conflict with 'Label already exists'
        error = urllib.error.HTTPError(
            url="https://gitlab.example.com/api/v4/projects/1/labels",
            code=409,
            msg="Conflict",
            hdrs={},
            fp=MagicMock(read=lambda: b'{"message":"Label already exists"}'),
        )
        mock_urlopen.side_effect = error

        success = self.provider.create_label("type::feature", "Feature description", "0366d6")
        self.assertTrue(success)

    @patch("urllib.request.urlopen")
    def test_create_label_server_error(self, mock_urlopen):
        error = urllib.error.HTTPError(
            url="https://gitlab.example.com/api/v4/projects/1/labels",
            code=500,
            msg="Internal Server Error",
            hdrs={},
            fp=MagicMock(read=lambda: b'{"message":"Internal error"}'),
        )
        mock_urlopen.side_effect = error

        self.provider.max_retries = 1
        success = self.provider.create_label("type::epic", "Epic description", "800080")
        self.assertFalse(success)

    def test_dry_run_mode(self):
        prov = GitLabV4LabelProvider(
            server_url="https://gitlab.example.com",
            project_id="group/proj",
            dry_run=True,
        )
        with patch("urllib.request.urlopen") as mock_urlopen:
            success = prov.create_label("type::user-story", "User story", "0e8a16")
            self.assertTrue(success)
            self.assertFalse(mock_urlopen.called)

    def test_offline_mode(self):
        prov = GitLabV4LabelProvider(
            server_url="https://gitlab.example.com",
            project_id="group/proj",
            offline=True,
        )
        with patch("urllib.request.urlopen") as mock_urlopen:
            success = prov.create_label("type::user-story", "User story", "0e8a16")
            self.assertTrue(success)
            self.assertFalse(mock_urlopen.called)


class TestGitHubCLILabelProvider(unittest.TestCase):
    def setUp(self):
        self.provider = GitHubCLILabelProvider(repo="owner/repo")

    @patch("subprocess.run")
    def test_create_label_success(self, mock_run):
        mock_run.return_value = MagicMock(returncode=0, stdout="", stderr="")

        success = self.provider.create_label("feature", "Feature capability", "0366d6")
        self.assertTrue(success)
        self.assertTrue(mock_run.called)
        cmd = mock_run.call_args[0][0]
        self.assertEqual(cmd[0], "gh")
        self.assertEqual(cmd[1], "label")
        self.assertEqual(cmd[2], "create")
        self.assertEqual(cmd[3], "feature")
        self.assertIn("--color", cmd)
        self.assertIn("0366d6", cmd)
        self.assertIn("--repo", cmd)
        self.assertIn("owner/repo", cmd)
        self.assertIn("--force", cmd)

    @patch("subprocess.run")
    def test_create_label_failure(self, mock_run):
        mock_run.return_value = MagicMock(returncode=1, stdout="", stderr="API Error")

        success = self.provider.create_label("feature", "Feature capability", "0366d6")
        self.assertFalse(success)

    def test_dry_run_mode(self):
        prov = GitHubCLILabelProvider(repo="owner/repo", dry_run=True)
        with patch("subprocess.run") as mock_run:
            success = prov.create_label("epic", "Epic", "800080")
            self.assertTrue(success)
            self.assertFalse(mock_run.called)

    def test_offline_mode(self):
        prov = GitHubCLILabelProvider(repo="owner/repo", offline=True)
        with patch("subprocess.run") as mock_run:
            success = prov.create_label("epic", "Epic", "800080")
            self.assertTrue(success)
            self.assertFalse(mock_run.called)


class TestBootstrapLabelsEndToEnd(unittest.TestCase):
    @patch("urllib.request.urlopen")
    def test_bootstrap_labels_gitlab_success(self, mock_urlopen):
        mock_resp = MagicMock()
        mock_resp.status = 201
        mock_resp.__enter__.return_value = mock_resp
        mock_urlopen.return_value = mock_resp

        labels = {
            "epic": "type::epic",
            "feature": "type::feature",
        }
        success = bootstrap_labels(
            provider_name="gitlab",
            labels=labels,
            project="test/project",
            gitlab_url="https://gitlab.example.com",
            token="glpat-token",
        )
        self.assertTrue(success)
        self.assertEqual(mock_urlopen.call_count, 2)

    @patch("subprocess.run")
    def test_bootstrap_labels_github_success(self, mock_run):
        mock_run.return_value = MagicMock(returncode=0, stdout="", stderr="")

        labels = {
            "epic": "epic",
            "feature": "feature",
        }
        success = bootstrap_labels(
            provider_name="github",
            labels=labels,
            repo="owner/repo",
        )
        self.assertTrue(success)
        self.assertEqual(mock_run.call_count, 2)

    def test_main_cli_dry_run_github(self):
        exit_code = main(["--provider", "github", "--repo", "owner/repo", "--dry-run"])
        self.assertEqual(exit_code, 0)

    def test_main_cli_dry_run_gitlab(self):
        exit_code = main(["--provider", "gitlab", "--project", "owner/proj", "--dry-run"])
        self.assertEqual(exit_code, 0)

    def test_main_cli_offline(self):
        exit_code = main(["--provider", "gitlab", "--project", "owner/proj", "--offline"])
        self.assertEqual(exit_code, 0)


if __name__ == "__main__":
    unittest.main()
