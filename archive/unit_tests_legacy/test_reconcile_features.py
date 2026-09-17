import os
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    DEFAULT_CODEBASE_RULES,
    DEFAULT_GITLAB_TRACKER_RULES,
    DEFAULT_JIRA_TRACKER_RULES,
    update_checklist_in_file,
    resolve_issue_on_tracker,
    is_already_resolved,
    get_resolved_label,
    main,
)


class TestReconcileFeatures(unittest.TestCase):
    """
    Unit and integration tests asserting that feature specifications with completed checklists
    invoke resolve_issue_on_tracker() and receive the status:fixed-resolved label.
    (Fixes Issue #66: Reconciler Feature Resolution)
    """

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "github",
                "dependency_regex": r"(-\s*\[\s*([ xX])\s*\]\s*(#|#\[|\#\s*)?([A-Za-z0-9\-]+))",
                "keys": {
                    "issue_id": "number",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                    "closed_state_value": "CLOSED",
                    "open_state_value": "OPEN",
                },
                "labels": {
                    "epic": "epic",
                    "feature": "feature",
                    "user_story": "user-story",
                    "use_case": "use-case",
                    "resolved": "status:fixed-resolved",
                },
                "close_comments": {
                    "epic": "Epic completed. All constituent features successfully delivered and verified.",
                    "feature": "Resolved. All acceptance criteria and verification tasks for feature '{title}' have been completed and verified.",
                    "user_story": "Resolved. All dependent features/tasks for BDD scenario '{title}' have been completed and verified.",
                    "use_case": "Resolved. All dependent user stories and features for use case '{title}' are completed.",
                },
                "commands": {
                    "resolve_issue": ["gh", "issue", "edit", "{number}", "--add-label", "{label}"],
                    "comment_issue": ["gh", "issue", "comment", "{number}", "--body", "{comment}"],
                },
            },
            "backlog_directories": {
                "epics": "docs/epics",
                "features": "docs/features",
                "user_stories": "docs/user-stories",
                "use_cases": "docs/use-cases",
                "schemas": "schema",
            },
        }

    def tearDown(self):
        self.temp_dir.cleanup()

    def _create_spec_file(self, rel_path: str, content: str) -> str:
        filepath = os.path.join(self.temp_dir.name, rel_path)
        os.makedirs(os.path.dirname(filepath), exist_ok=True)
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)
        return filepath

    def test_feature_close_comment_templates_defined_across_all_providers(self):
        """
        Verify that feature close comment templates are defined in GitHub, GitLab, and Jira default tracker rules.
        """
        github_close_comments = DEFAULT_CODEBASE_RULES.get("tracker_rules", {}).get("close_comments", {})
        gitlab_close_comments = DEFAULT_GITLAB_TRACKER_RULES.get("close_comments", {})
        jira_close_comments = DEFAULT_JIRA_TRACKER_RULES.get("close_comments", {})

        self.assertIn("feature", github_close_comments, "GitHub tracker rules missing 'feature' close comment")
        self.assertIn("feature", gitlab_close_comments, "GitLab tracker rules missing 'feature' close comment")
        self.assertIn("feature", jira_close_comments, "Jira tracker rules missing 'feature' close comment")

        # Verify formatting with {title}
        sample_title = "Autonomous Flight Controller"
        self.assertIn(sample_title, github_close_comments["feature"].format(title=sample_title))
        self.assertIn(sample_title, gitlab_close_comments["feature"].format(title=sample_title))
        self.assertIn(sample_title, jira_close_comments["feature"].format(title=sample_title))

    def test_feature_completed_checklist_invokes_resolve_issue_on_tracker(self):
        """
        Assert that feature specifications with completed checklists invoke resolve_issue_on_tracker()
        and receive the status:fixed-resolved label.
        """
        content = (
            "---\n"
            "issue_id: 201\n"
            "title: Feature 201: Attitude Determination\n"
            "type: feature\n"
            "---\n\n"
            "# Feature 201: Attitude Determination\n\n"
            "## Acceptance Criteria & Tasks\n"
            "- [ ] #101 - Sensor fusion algorithm\n"
            "- [ ] #102 - EKF state estimation\n"
        )
        spec_file = self._create_spec_file("docs/features/feat-01.md", content)

        issue_dict = {
            101: {"number": 101, "title": "Sensor fusion algorithm", "state": "CLOSED", "labels": []},
            102: {"number": 102, "title": "EKF state estimation", "state": "CLOSED", "labels": []},
            201: {"number": 201, "title": "Feature 201: Attitude Determination", "state": "OPEN", "labels": [{"name": "feature"}]},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertTrue(completed, "Feature with all dependencies closed must be marked completed")
        self.assertIn("- [x] #101", updated_content)
        self.assertIn("- [x] #102", updated_content)

        mock_provider = MagicMock()
        title = "Feature 201: Attitude Determination"
        feature_comment_template = self.rules["tracker_rules"]["close_comments"]["feature"]

        if completed and not is_already_resolved(issue_dict[201], self.rules):
            resolve_issue_on_tracker(
                201,
                feature_comment_template.format(title=title),
                rules=self.rules,
                provider_adapter=mock_provider,
            )
            issue_dict[201].setdefault("labels", []).append({"name": get_resolved_label(self.rules)})

        mock_provider.add_label.assert_called_once_with(201, "status:fixed-resolved")
        mock_provider.comment_issue.assert_called_once_with(
            201,
            "Resolved. All acceptance criteria and verification tasks for feature 'Feature 201: Attitude Determination' have been completed and verified.",
        )
        resolved_labels = [l.get("name") for l in issue_dict[201]["labels"]]
        self.assertIn("status:fixed-resolved", resolved_labels)

    def test_feature_open_checklist_does_not_invoke_resolve_issue_on_tracker(self):
        """
        Assert that feature specifications with open checklist items do NOT invoke resolve_issue_on_tracker().
        """
        content = (
            "---\n"
            "issue_id: 202\n"
            "title: Feature 202: Trajectory Planning\n"
            "type: feature\n"
            "---\n\n"
            "# Feature 202: Trajectory Planning\n\n"
            "## Acceptance Criteria & Tasks\n"
            "- [x] #101 - Core Waypoint Generator (Closed)\n"
            "- [ ] #103 - Obstacle Avoidance Optimizer (Open)\n"
        )
        spec_file = self._create_spec_file("docs/features/feat-02.md", content)

        issue_dict = {
            101: {"number": 101, "title": "Core Waypoint Generator", "state": "CLOSED", "labels": []},
            103: {"number": 103, "title": "Obstacle Avoidance Optimizer", "state": "OPEN", "labels": []},
            202: {"number": 202, "title": "Feature 202: Trajectory Planning", "state": "OPEN", "labels": [{"name": "feature"}]},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Feature with open checklist items must NOT be marked completed")

        mock_provider = MagicMock()
        if completed and not is_already_resolved(issue_dict[202], self.rules):
            resolve_issue_on_tracker(
                202,
                "Feature completed.",
                rules=self.rules,
                provider_adapter=mock_provider,
            )

        mock_provider.add_label.assert_not_called()
        mock_provider.comment_issue.assert_not_called()

    def test_reconcile_main_processes_feature_resolution(self):
        """
        Test that running reconcile_backlog main loop processes Features directory,
        calls update_checklist_in_file, and marks completed features resolved on the tracker.
        """
        # Create directory layout
        feat_content = (
            "---\n"
            "issue_id: 301\n"
            "title: Navigation Filter\n"
            "type: feature\n"
            "---\n\n"
            "# Feature: Navigation Filter\n\n"
            "## Acceptance Criteria\n"
            "- [ ] #110 - Kalman filter initialization\n"
        )
        self._create_spec_file("docs/features/feat-01.md", feat_content)
        self._create_spec_file("docs/epics/.gitkeep", "")
        self._create_spec_file("docs/user-stories/.gitkeep", "")
        self._create_spec_file("docs/use-cases/.gitkeep", "")

        issues = [
            {"number": 110, "title": "Kalman filter initialization", "state": "CLOSED", "labels": []},
            {"number": 301, "title": "Navigation Filter", "state": "OPEN", "labels": [{"name": "feature"}]},
        ]

        mock_adapter = MagicMock()
        mock_adapter.create_label.return_value = True
        mock_adapter.add_label.return_value = True
        mock_adapter.comment_issue.return_value = True
        mock_adapter.edit_issue.return_value = True
        mock_adapter.edit_issue_title.return_value = True

        test_args = [
            "reconcile_backlog.py",
            os.path.join(self.temp_dir.name, "docs"),
            "--offline",
        ]

        with patch.object(sys, "argv", test_args), \
             patch("reconcile_backlog.resolve_linter_script", return_value=None), \
             patch("reconcile_backlog.detect_tracker_provider", return_value="github"), \
             patch("reconcile_backlog.load_codebase_rules", return_value=self.rules), \
             patch("reconcile_backlog.create_tracker_provider", return_value=mock_adapter), \
             patch("reconcile_backlog.get_all_issues", return_value=issues), \
             patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_upstream_repository", return_value="gintatkinson/DEAP01-spec-core"):
            main()

        # Check that resolve_issue_on_tracker was invoked for feature #301
        mock_adapter.add_label.assert_any_call(301, "status:fixed-resolved")
        mock_adapter.comment_issue.assert_called_once_with(
            301,
            "Resolved. All acceptance criteria and verification tasks for feature 'Navigation Filter' have been completed and verified.",
        )

        # Check that the local feature file checklist was updated to [x]
        with open(os.path.join(self.temp_dir.name, "docs/features/feat-01.md"), "r", encoding="utf-8") as f:
            saved_content = f.read()
        self.assertIn("- [x] #110", saved_content)


if __name__ == "__main__":
    unittest.main()
