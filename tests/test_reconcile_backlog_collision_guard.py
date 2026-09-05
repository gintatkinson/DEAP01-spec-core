import io
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
    is_issue_type_compatible,
    get_type_for_structural_label,
    resolve_spec_issue_number,
    DEFAULT_CODEBASE_RULES,
    DEFAULT_GITLAB_TRACKER_RULES,
    DEFAULT_JIRA_TRACKER_RULES,
)


class TestReconcileBacklogCollisionGuard(unittest.TestCase):
    """
    Regression tests for Issue #221:
    reconcile_backlog.py overwrote live Feature issues via local-ordinal iid collision.
    """

    def setUp(self):
        self.github_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "github",
                "keys": {
                    "issue_id": "number",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                },
                "labels": {
                    "epic": "epic",
                    "feature": "feature",
                    "user_story": "user-story",
                    "use_case": "use-case",
                    "resolved": "status:fixed-resolved",
                },
            },
        }

        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "gitlab",
                "keys": {
                    "issue_id": "iid",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                },
                "labels": {
                    "epic": "type::epic",
                    "feature": "type::feature",
                    "user_story": "type::user-story",
                    "use_case": "type::use-case",
                    "resolved": "status::fixed-resolved",
                },
            },
        }

        self.jira_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {
                "provider": "jira",
                "keys": {
                    "issue_id": "key",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                },
                "labels": {
                    "epic": "type::epic",
                    "feature": "type::feature",
                    "user_story": "type::user-story",
                    "use_case": "type::use-case",
                    "resolved": "status::fixed-resolved",
                },
            },
        }

    def test_get_type_for_structural_label(self):
        self.assertEqual(get_type_for_structural_label("epic", self.github_rules), "epic")
        self.assertEqual(get_type_for_structural_label("type::epic", self.gitlab_rules), "epic")
        self.assertEqual(get_type_for_structural_label("feature", self.github_rules), "feature")
        self.assertEqual(get_type_for_structural_label("type::feature", self.gitlab_rules), "feature")
        self.assertEqual(get_type_for_structural_label("user-story", self.github_rules), "user_story")
        self.assertEqual(get_type_for_structural_label("type::user-story", self.gitlab_rules), "user_story")
        self.assertEqual(get_type_for_structural_label("use-case", self.github_rules), "use_case")
        self.assertEqual(get_type_for_structural_label("type::use-case", self.gitlab_rules), "use_case")
        # Non-structural labels
        self.assertIsNone(get_type_for_structural_label("status:fixed-resolved", self.github_rules))
        self.assertIsNone(get_type_for_structural_label("bug", self.github_rules))
        self.assertIsNone(get_type_for_structural_label("enhancement", self.github_rules))
        self.assertIsNone(get_type_for_structural_label("", self.github_rules))
        self.assertIsNone(get_type_for_structural_label(None, self.github_rules))

    def test_is_issue_type_compatible_gitlab(self):
        epic_record = {
            "iid": 26,
            "title": "System Architecture",
            "labels": [{"name": "type::epic"}],
        }
        feature_record = {
            "iid": 1,
            "title": "Monitor RTA Safety Net",
            "labels": [{"name": "type::feature"}],
        }
        story_record = {
            "iid": 50,
            "title": "Engage autopilot",
            "labels": [{"name": "type::user-story"}],
        }
        usecase_record = {
            "iid": 80,
            "title": "Autonomous waypoint navigation",
            "labels": [{"name": "type::use-case"}],
        }

        # Compatible checks
        self.assertTrue(is_issue_type_compatible(epic_record, "Epic", self.gitlab_rules))
        self.assertTrue(is_issue_type_compatible(feature_record, "Feature", self.gitlab_rules))
        self.assertTrue(is_issue_type_compatible(story_record, "User Story", self.gitlab_rules))
        self.assertTrue(is_issue_type_compatible(usecase_record, "Use Case", self.gitlab_rules))

        # Incompatible checks (detects conflicting structural labels)
        self.assertFalse(is_issue_type_compatible(feature_record, "Epic", self.gitlab_rules))
        self.assertFalse(is_issue_type_compatible(epic_record, "Feature", self.gitlab_rules))
        self.assertFalse(is_issue_type_compatible(story_record, "Feature", self.gitlab_rules))
        self.assertFalse(is_issue_type_compatible(usecase_record, "Epic", self.gitlab_rules))

    def test_is_issue_type_compatible_github_string_labels(self):
        feature_record = {
            "number": 1,
            "title": "Monitor RTA Safety Net",
            "labels": ["feature", "status:fixed-resolved"],
        }
        self.assertTrue(is_issue_type_compatible(feature_record, "Feature", self.github_rules))
        self.assertFalse(is_issue_type_compatible(feature_record, "Epic", self.github_rules))

    def test_is_issue_type_compatible_empty_or_none(self):
        unlabeled_record = {"number": 1, "title": "Generic task", "labels": []}
        self.assertTrue(is_issue_type_compatible(unlabeled_record, "Epic", self.github_rules))
        self.assertTrue(is_issue_type_compatible(None, "Epic", self.github_rules))

    def test_resolve_spec_issue_number_normal_match(self):
        spec_content = (
            "---\n"
            "title: System Architecture\n"
            "issue_id: 26\n"
            "---\n"
            "# Epic: System Architecture\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            issue_dict = {
                26: {
                    "iid": 26,
                    "title": "System Architecture",
                    "labels": [{"name": "type::epic"}],
                },
                "26": {
                    "iid": 26,
                    "title": "System Architecture",
                    "labels": [{"name": "type::epic"}],
                },
            }
            title_map = {"system architecture": 26}
            claimed = {}

            resolved = resolve_spec_issue_number(
                filepath=temp_path,
                title="System Architecture",
                title_map=title_map,
                issue_dict=issue_dict,
                rules=self.gitlab_rules,
                item_type="Epic",
                claimed=claimed,
            )

            self.assertEqual(resolved, 26)
            self.assertEqual(claimed.get("26"), temp_path)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_resolve_spec_issue_number_collision_auto_reconciles_to_true_epic(self):
        """
        Test that when local Epic has issue_id: 1 (colliding with Feature #1),
        but remote Epic is issue #26 with matching normalized title:
        1. The collision on issue #1 is detected.
        2. Epic-01 is auto-reconciled to issue #26 via normalized title match.
        3. Feature #1 is NOT returned.
        """
        spec_content = (
            "---\n"
            "title: System Architecture\n"
            "issue_id: 1\n"
            "---\n"
            "# Epic: System Architecture\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            issue_dict = {
                1: {
                    "iid": 1,
                    "title": "Monitor RTA Safety Net",
                    "labels": [{"name": "type::feature"}],
                },
                "1": {
                    "iid": 1,
                    "title": "Monitor RTA Safety Net",
                    "labels": [{"name": "type::feature"}],
                },
                26: {
                    "iid": 26,
                    "title": "System Architecture",
                    "labels": [{"name": "type::epic"}],
                },
                "26": {
                    "iid": 26,
                    "title": "System Architecture",
                    "labels": [{"name": "type::epic"}],
                },
            }
            title_map = {"system architecture": 26}
            claimed = {}

            with io.StringIO() as buf, patch("sys.stdout", buf):
                resolved = resolve_spec_issue_number(
                    filepath=temp_path,
                    title="System Architecture",
                    title_map=title_map,
                    issue_dict=issue_dict,
                    rules=self.gitlab_rules,
                    item_type="Epic",
                    claimed=claimed,
                )
                stdout_val = buf.getvalue()

            self.assertEqual(resolved, 26)
            self.assertIn("[Notice]", stdout_val)
            self.assertIn("Auto-reconciled to true tracker issue #26", stdout_val)
            self.assertIn("#221", stdout_val)
            self.assertEqual(claimed.get("26"), temp_path)
            self.assertNotIn("1", claimed)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_resolve_spec_issue_number_collision_fails_closed_when_no_title_match(self):
        """
        Test that when local Epic has issue_id: 1 colliding with Feature #1,
        and no Epic with a matching title exists on the tracker,
        the reconciler fails closed with exit code 1 and aborts before overwriting Feature #1.
        """
        spec_content = (
            "---\n"
            "title: System Architecture\n"
            "issue_id: 1\n"
            "---\n"
            "# Epic: System Architecture\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            issue_dict = {
                1: {
                    "iid": 1,
                    "title": "Monitor RTA Safety Net",
                    "labels": [{"name": "type::feature"}],
                },
                "1": {
                    "iid": 1,
                    "title": "Monitor RTA Safety Net",
                    "labels": [{"name": "type::feature"}],
                },
            }
            title_map = {}  # No matching Epic exists on tracker
            claimed = {}

            with io.StringIO() as stderr_buf, patch("sys.stderr", stderr_buf):
                with self.assertRaises(SystemExit) as cm:
                    resolve_spec_issue_number(
                        filepath=temp_path,
                        title="System Architecture",
                        title_map=title_map,
                        issue_dict=issue_dict,
                        rules=self.gitlab_rules,
                        item_type="Epic",
                        claimed=claimed,
                    )
                stderr_val = stderr_buf.getvalue()

            self.assertEqual(cm.exception.code, 1)
            self.assertIn("[FATAL]", stderr_val)
            self.assertIn("#221", stderr_val)
            self.assertIn("Refusing to overwrite tracker issue #1", stderr_val)
            self.assertEqual(claimed, {})
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_resolve_spec_issue_number_title_mismatch_collision(self):
        """
        Test that even if the issue label is not explicitly conflicting,
        a significant title mismatch between local spec and remote issue triggers collision handling.
        """
        spec_content = (
            "---\n"
            "title: Telemetry Pipeline\n"
            "issue_id: 1\n"
            "---\n"
            "# Feature: Telemetry Pipeline\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            issue_dict = {
                1: {
                    "number": 1,
                    "title": "Completely Unrelated Feature",
                    "labels": ["feature"],
                },
                "1": {
                    "number": 1,
                    "title": "Completely Unrelated Feature",
                    "labels": ["feature"],
                },
                15: {
                    "number": 15,
                    "title": "Telemetry Pipeline",
                    "labels": ["feature"],
                },
                "15": {
                    "number": 15,
                    "title": "Telemetry Pipeline",
                    "labels": ["feature"],
                },
            }
            title_map = {"telemetry pipeline": 15}
            claimed = {}

            with io.StringIO() as stdout_buf, patch("sys.stdout", stdout_buf):
                resolved = resolve_spec_issue_number(
                    filepath=temp_path,
                    title="Telemetry Pipeline",
                    title_map=title_map,
                    issue_dict=issue_dict,
                    rules=self.github_rules,
                    item_type="Feature",
                    claimed=claimed,
                )
                out_val = stdout_buf.getvalue()

            self.assertEqual(resolved, 15)
            self.assertIn("title mismatch", out_val)
            self.assertIn("#221", out_val)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)


if __name__ == "__main__":
    unittest.main()
