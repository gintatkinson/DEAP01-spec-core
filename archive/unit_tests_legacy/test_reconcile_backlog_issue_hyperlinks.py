import os
import sys
import tempfile
import unittest
from unittest.mock import patch

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    get_issue_web_url,
    expand_relative_links_for_tracker,
    reconcile_epic_checklists,
    update_checklist_in_file,
)


class TestReconcileBacklogIssueHyperlinks(unittest.TestCase):
    """
    Test suite for Issue #224: Explicit Hyperlinking for Task Lists.
    Verifies that leading issue references in task lists are expanded into
    provider-aware explicit markdown hyperlinks rather than left as bare #N tokens.
    """

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.workspace_dir = self.temp_dir.name

        # Create mock file structure
        self.feature_file = os.path.join(self.workspace_dir, "docs", "features", "feat-01-auth.md")
        os.makedirs(os.path.dirname(self.feature_file), exist_ok=True)
        with open(self.feature_file, "w", encoding="utf-8") as f:
            f.write("# Feature 01 Auth\n")

        self.epic_file = os.path.join(self.workspace_dir, "docs", "epics", "epic-01.md")
        os.makedirs(os.path.dirname(self.epic_file), exist_ok=True)

        self.github_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "github",
                "issue_id_placeholder": "#[IssueID]",
            }
        }

        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/uav-008"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.com",
                "issue_id_placeholder": "#[IssueID]",
            }
        }

        self.jira_rules = {
            "meta": {"upstream_repository": "corp/platform"},
            "tracker_rules": {
                "provider": "jira",
                "server_url": "https://jira.example.com",
                "project_key": "DEAP",
                "issue_id_placeholder": "#[IssueID]",
            }
        }

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_get_issue_web_url_github(self):
        url = get_issue_web_url(101, rules=self.github_rules, workspace_dir=self.workspace_dir)
        self.assertEqual(url, "https://github.com/gintatkinson/DEAP01-spec-core/issues/101")

    def test_get_issue_web_url_gitlab(self):
        url = get_issue_web_url(26, rules=self.gitlab_rules, workspace_dir=self.workspace_dir)
        self.assertEqual(url, "https://gitlab.com/gintatkinson/uav-008/-/issues/26")

    def test_get_issue_web_url_gitlab_custom_server(self):
        custom_rules = {
            "meta": {"upstream_repository": "internal-group/sub-project"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.internal.corp",
            }
        }
        url = get_issue_web_url(42, rules=custom_rules, workspace_dir=self.workspace_dir)
        self.assertEqual(url, "https://gitlab.internal.corp/internal-group/sub-project/-/issues/42")

    def test_get_issue_web_url_jira(self):
        url = get_issue_web_url(101, rules=self.jira_rules, workspace_dir=self.workspace_dir)
        self.assertEqual(url, "https://jira.example.com/browse/DEAP-101")

    def test_get_issue_web_url_empty_or_zero(self):
        self.assertEqual(get_issue_web_url(None, rules=self.github_rules, workspace_dir=self.workspace_dir), "")
        self.assertEqual(get_issue_web_url("", rules=self.github_rules, workspace_dir=self.workspace_dir), "")
        self.assertEqual(get_issue_web_url(0, rules=self.github_rules, workspace_dir=self.workspace_dir), "")

    def test_expand_relative_links_hyperlinks_task_list_github(self):
        content = (
            "## Requirements\n\n"
            "- [ ] #101 - [Feature Auth](docs/features/feat-01-auth.md) (authentication requirement)\n"
            "- [x] #102 - [Feature Core](docs/features/feat-01-auth.md) (core logic)\n"
        )
        mock_remote_info = {
            "raw": "https://github.com/gintatkinson/DEAP01-spec-core.git",
            "is_gitlab": False,
            "project_path": "gintatkinson/DEAP01-spec-core",
            "server_url": "https://github.com",
            "host": "github.com"
        }
        with patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote_info):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=self.epic_file,
                rules=self.github_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn(
            "- [ ] [#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101) - [Feature Auth](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/features/feat-01-auth.md)",
            expanded
        )
        self.assertIn(
            "- [x] [#102](https://github.com/gintatkinson/DEAP01-spec-core/issues/102) - [Feature Core](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/features/feat-01-auth.md)",
            expanded
        )

    def test_expand_relative_links_hyperlinks_task_list_gitlab(self):
        content = (
            "## Requirements\n\n"
            "- [ ] #26 - [Operate Onboard Computer Autopilot](docs/features/feat-01-auth.md) (flight control)\n"
        )
        mock_remote_info = {
            "raw": "https://gitlab.com/gintatkinson/uav-008.git",
            "is_gitlab": True,
            "project_path": "gintatkinson/uav-008",
            "server_url": "https://gitlab.com",
            "host": "gitlab.com"
        }
        with patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote_info):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=self.epic_file,
                rules=self.gitlab_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn(
            "- [ ] [#26](https://gitlab.com/gintatkinson/uav-008/-/issues/26) - [Operate Onboard Computer Autopilot](https://gitlab.com/gintatkinson/uav-008/-/blob/main/docs/features/feat-01-auth.md)",
            expanded
        )

    def test_expand_relative_links_idempotent_no_double_wrapping(self):
        content = (
            "- [ ] [#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101) - [Feature Auth](docs/features/feat-01-auth.md)\n"
        )
        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=self.epic_file,
                rules=self.github_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn("[#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101)", expanded)
        self.assertNotIn("[[#101]", expanded)

    def test_expand_relative_links_leaves_placeholders_intact(self):
        content = (
            "- [ ] #[IssueID] - [Pending Feature](docs/features/feat-01-auth.md) (semantic linkage)\n"
        )
        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=self.epic_file,
                rules=self.github_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn("- [ ] #[IssueID] - [Pending Feature]", expanded)

    def test_reconcile_epic_checklists_synthesizes_explicit_issue_hyperlinks(self):
        epic_content = (
            "---\n"
            "title: 'Flight Control Subsystem'\n"
            "type: epic\n"
            "---\n\n"
            "# Epic: Flight Control Subsystem\n\n"
            "## 2. Requirements & Checklist\n"
            "- [ ] #[IssueID] - [Feature Auth](../features/feat-01-auth.md) (semantic linkage)\n\n"
            "### Associated Use Cases & User Stories\n\n"
            "#### Associated Use Cases\n\n"
            "#### Associated User Stories\n"
        )
        with open(self.epic_file, "w", encoding="utf-8") as f:
            f.write(epic_content)

        child_features = [("feat-01-auth", "Feature Auth")]
        feature_titles = {"auth": 101}

        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            reconcile_epic_checklists(
                self.epic_file,
                child_features=child_features,
                child_stories=[],
                child_usecases=[],
                epic_titles={},
                feature_titles=feature_titles,
                story_titles={},
                usecase_titles={},
                rules=self.github_rules
            )

        with open(self.epic_file, "r", encoding="utf-8") as f:
            result = f.read()

        self.assertIn(
            "- [ ] [#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101) - [Feature Auth]",
            result
        )

    def test_update_checklist_in_file_with_hyperlinked_issues(self):
        content = (
            "- [ ] [#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101) - [Feature Auth](...)\n"
            "- [ ] [#102](https://github.com/gintatkinson/DEAP01-spec-core/issues/102) - [Feature Core](...)\n"
        )
        with open(self.epic_file, "w", encoding="utf-8") as f:
            f.write(content)

        issue_dict = {
            101: {"number": 101, "state": "CLOSED", "labels": []},
            102: {"number": 102, "state": "OPEN", "labels": []}
        }

        updated, all_closed = update_checklist_in_file(self.epic_file, issue_dict, rules=self.github_rules)

        self.assertIn("- [x] [#101](https://github.com/gintatkinson/DEAP01-spec-core/issues/101)", updated)
        self.assertIn("- [ ] [#102](https://github.com/gintatkinson/DEAP01-spec-core/issues/102)", updated)
        self.assertFalse(all_closed)


if __name__ == "__main__":
    unittest.main()
