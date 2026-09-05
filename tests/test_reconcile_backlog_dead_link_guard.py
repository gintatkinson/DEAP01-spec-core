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
    expand_relative_links_for_tracker,
    reconcile_epic_checklists,
)


class TestReconcileBacklogDeadLinkGuard(unittest.TestCase):
    """
    Test suite for Issue #222: Dead Link & Unregistered Reference Publish Guard.
    Verifies that:
    1. expand_relative_links_for_tracker guards against generating 404 blob URLs
       when target files do not exist in the workspace.
    2. expand_relative_links_for_tracker and reconcile_epic_checklists guard against
       publishing unverified/assumed issue numbers by reverting them to placeholders.
    """

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.workspace_dir = self.temp_dir.name

        # Create existing feature file
        self.existing_feat = os.path.join(self.workspace_dir, "docs", "features", "feat-16-model-avenger5.md")
        os.makedirs(os.path.dirname(self.existing_feat), exist_ok=True)
        with open(self.existing_feat, "w", encoding="utf-8") as f:
            f.write("# Feature 16 Model Avenger5\n")

        self.epic_file = os.path.join(self.workspace_dir, "docs", "epics", "epic-02.md")
        os.makedirs(os.path.dirname(self.epic_file), exist_ok=True)

        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/uav-008"},
            "tracker_rules": {
                "provider": "gitlab",
                "server_url": "https://gitlab.com",
                "issue_id_placeholder": "#[IssueID]",
            }
        }

        self.github_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "github",
                "issue_id_placeholder": "#[IssueID]",
            }
        }

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_expand_relative_links_retains_missing_file_links(self):
        # feat-16 exists on disk, feat-01 does NOT exist on disk
        content = (
            "## Requirements\n\n"
            "- [ ] #1 - [Model Avenger5](docs/features/feat-16-model-avenger5.md) (air vehicle)\n"
            "- [ ] #101 - [Execute Arming Procedure](docs/features/feat-01-execute-arming.md) (arming logic)\n"
            "- [ ] #102 - [Missing Story](../user-stories/us-01-missing.md) (story logic)\n"
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

        # Existing file expands to blob URL
        self.assertIn(
            "[Model Avenger5](https://gitlab.com/gintatkinson/uav-008/-/blob/main/docs/features/feat-16-model-avenger5.md)",
            expanded
        )
        # Non-existing files retain relative links (no dead 404 blob URLs)
        self.assertIn("[Execute Arming Procedure](docs/features/feat-01-execute-arming.md)", expanded)
        self.assertIn("[Missing Story](../user-stories/us-01-missing.md)", expanded)
        self.assertNotIn("https://gitlab.com/gintatkinson/uav-008/-/blob/main/docs/features/feat-01-execute-arming.md", expanded)
        self.assertNotIn("https://gitlab.com/gintatkinson/uav-008/-/blob/main/docs/user-stories/us-01-missing.md", expanded)

    def test_expand_relative_links_reverts_unregistered_issues(self):
        content = (
            "- [ ] #1 - [Model Avenger5](docs/features/feat-16-model-avenger5.md)\n"
            "- [ ] #999 - [Unregistered Feature](docs/features/feat-16-model-avenger5.md)\n"
        )
        known_issues = {1}
        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=self.epic_file,
                rules=self.gitlab_rules,
                workspace_dir=self.workspace_dir,
                known_issue_ids=known_issues
            )

        # Issue 1 is known and registered -> hyperlinked
        self.assertIn("[#1](https://gitlab.com/gintatkinson/uav-008/-/issues/1)", expanded)
        # Issue 999 is unknown/unregistered -> reverted to #[IssueID]
        self.assertIn("- [ ] #[IssueID] - [Unregistered Feature]", expanded)
        self.assertNotIn("#999", expanded)

    def test_reconcile_epic_checklists_reverts_assumed_numbers_for_unregistered_siblings(self):
        epic_content = (
            "---\n"
            "title: 'UAS Safety Subsystem'\n"
            "type: epic\n"
            "---\n\n"
            "# Epic: UAS Safety Subsystem\n\n"
            "## 2. Requirements & Checklist\n"
            "- [ ] #101 - [Unregistered Feature](../features/feat-01-unregistered.md) (semantic linkage)\n"
            "- [ ] #16 - [Model Avenger5](../features/feat-16-model-avenger5.md) (semantic linkage)\n\n"
            "### Associated Use Cases & User Stories\n\n"
            "#### Associated Use Cases\n\n"
            "#### Associated User Stories\n"
        )
        with open(self.epic_file, "w", encoding="utf-8") as f:
            f.write(epic_content)

        child_features = [
            ("feat-01-unregistered", "Unregistered Feature"),
            ("feat-16-model-avenger5", "Model Avenger5"),
        ]
        # Only feat-16 is registered on tracker as issue 16
        feature_titles = {"model avenger5": 16}

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
                rules=self.gitlab_rules
            )

        with open(self.epic_file, "r", encoding="utf-8") as f:
            result = f.read()

        # Registered feature gets explicit hyperlink
        self.assertIn("[#16](https://gitlab.com/gintatkinson/uav-008/-/issues/16)", result)
        # Unregistered feature reverts from assumed #101 to #[IssueID]
        self.assertIn("- [ ] #[IssueID] - [Unregistered Feature]", result)
        self.assertNotIn("#101", result)


if __name__ == "__main__":
    unittest.main()
