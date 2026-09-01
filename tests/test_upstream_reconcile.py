# Copyright Gint Atkinson, gint.atkinson@gmail.com

import os
import sys
import json
import tempfile
import unittest
from unittest.mock import patch, MagicMock, call

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from reconcile_backlog import (
    is_upstream_repository,
    extract_test_targets_from_text,
    load_compiler_backlog_manifest,
    execute_test_target,
    reconcile_upstream_compiler_backlog,
)


class TestUpstreamReconcile(unittest.TestCase):
    """
    Test suite for Issue #68: Clean Landing Zone Reconciler.
    Verifies that upstream repositories (which have clean landing zones with zero concrete specs)
    can reconcile their compiler backlog issues against test targets via manifest or annotations.
    """

    def setUp(self):
        self.workspace_dir = tempfile.mkdtemp()
        self.github_rules = {
            "meta": {
                "upstream_repository": "gintatkinson/DEAP01-spec-core",
                "repository_type": "UPSTREAM_SPEC_CORE_COMPILER",
            },
            "tracker_rules": {
                "provider": "github",
                "keys": {
                    "issue_id": "number",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                    "closed_state_value": "CLOSED",
                    "open_state_value": "OPEN",
                },
                "labels": {
                    "resolved": "status:fixed-resolved",
                },
                "close_comments": {
                    "compiler": "Resolved. Compiler backlog test target(s) '{test_targets}' passed successfully.",
                },
            },
        }

    def tearDown(self):
        import shutil
        if os.path.exists(self.workspace_dir):
            shutil.rmtree(self.workspace_dir, ignore_errors=True)

    def test_is_upstream_repository_detection(self):
        """Verify is_upstream_repository correctly detects upstream sentinel marker and environment."""
        # Case 1: Downstream (no marker, no env)
        with patch.dict(os.environ, {}, clear=True):
            self.assertFalse(is_upstream_repository(self.workspace_dir))

        # Case 2: Upstream marker directory .pipeline/upstream
        upstream_marker = os.path.join(self.workspace_dir, ".pipeline", "upstream")
        os.makedirs(upstream_marker, exist_ok=True)
        self.assertTrue(is_upstream_repository(self.workspace_dir))

        # Case 3: Environment variable override
        import shutil
        shutil.rmtree(upstream_marker, ignore_errors=True)
        with patch.dict(os.environ, {"DEAP_REPOSITORY_TYPE": "UPSTREAM_SPEC_CORE_COMPILER"}):
            self.assertTrue(is_upstream_repository(self.workspace_dir))

    def test_extract_test_targets_from_annotations(self):
        """Verify test targets can be extracted from various annotation patterns in issue body."""
        # HTML comment single target
        text1 = "Some description\n<!-- test-target: tests/test_upstream_reconcile.py -->\nFooter"
        self.assertEqual(extract_test_targets_from_text(text1), ["tests/test_upstream_reconcile.py"])

        # HTML comment multiple comma-separated targets
        text2 = "<!-- test-targets: tests/test_a.py, tests/test_b.py -->"
        self.assertEqual(extract_test_targets_from_text(text2), ["tests/test_a.py", "tests/test_b.py"])

        # Key-value pattern
        text3 = "Fix Issue #68\nTest-Target: tests/test_upstream_reconcile.py\nMore notes"
        self.assertEqual(extract_test_targets_from_text(text3), ["tests/test_upstream_reconcile.py"])

        # Key-value plural pattern
        text4 = "Test-Targets: tests/test_1.py, tests/test_2.py"
        self.assertEqual(extract_test_targets_from_text(text4), ["tests/test_1.py", "tests/test_2.py"])

        # Markdown section with list
        text5 = "## Test Targets\n- `tests/test_upstream_reconcile.py`\n- tests/test_reconcile_backlog.py\n"
        self.assertEqual(
            extract_test_targets_from_text(text5),
            ["tests/test_upstream_reconcile.py", "tests/test_reconcile_backlog.py"],
        )

        # Empty / non-matching text
        text6 = "Regular issue description with no test target annotations."
        self.assertEqual(extract_test_targets_from_text(text6), [])

    def test_load_compiler_backlog_manifest(self):
        """Verify loading manifest from dict JSON, list JSON, and rules dict."""
        manifest_file = os.path.join(self.workspace_dir, "compiler_manifest.json")

        # Dict format
        dict_data = {
            "68": {
                "test_targets": ["tests/test_upstream_reconcile.py"],
                "title": "Clean Landing Zone Reconciler",
            },
            "67": ["tests/test_canonical_templates.py"],
        }
        with open(manifest_file, "w", encoding="utf-8") as f:
            json.dump(dict_data, f)

        loaded_dict = load_compiler_backlog_manifest(manifest_path=manifest_file)
        self.assertIn("68", loaded_dict)
        self.assertEqual(loaded_dict["68"]["test_targets"], ["tests/test_upstream_reconcile.py"])
        self.assertIn("67", loaded_dict)
        self.assertEqual(loaded_dict["67"]["test_targets"], ["tests/test_canonical_templates.py"])

        # List format
        list_data = [
            {
                "issue_id": 68,
                "test_targets": ["tests/test_upstream_reconcile.py"],
                "title": "Clean Landing Zone Reconciler",
            },
            {
                "number": 67,
                "test_target": "tests/test_canonical_templates.py",
            },
        ]
        with open(manifest_file, "w", encoding="utf-8") as f:
            json.dump(list_data, f)

        loaded_list = load_compiler_backlog_manifest(manifest_path=manifest_file)
        self.assertIn("68", loaded_list)
        self.assertEqual(loaded_list["68"]["test_targets"], ["tests/test_upstream_reconcile.py"])
        self.assertIn("67", loaded_list)
        self.assertEqual(loaded_list["67"]["test_targets"], ["tests/test_canonical_templates.py"])

        # From rules dict
        rules = {
            "compiler_backlog_manifest": {
                "68": {"test_targets": ["tests/test_upstream_reconcile.py"]}
            }
        }
        loaded_rules = load_compiler_backlog_manifest(rules=rules)
        self.assertIn("68", loaded_rules)

    def test_reconcile_upstream_compiler_backlog_with_manifest_passing(self):
        """Verify passing test target transitions compiler issue to status:fixed-resolved."""
        mock_provider = MagicMock()
        mock_provider.list_issues.return_value = [
            {
                "number": 68,
                "title": "Clean Landing Zone Reconciler",
                "state": "OPEN",
                "labels": [{"name": "enhancement"}],
                "body": "Reconcile clean upstream compiler backlog.",
            }
        ]

        manifest = {
            "68": {
                "test_targets": ["tests/test_upstream_reconcile.py"],
            }
        }

        mock_test_executor = MagicMock(return_value=(True, "OK (ran 5 tests)"))

        result = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            manifest=manifest,
            test_executor=mock_test_executor,
        )

        self.assertEqual(result["processed"], 1)
        self.assertEqual(result["passed"], 1)
        self.assertEqual(result["failed"], 0)
        self.assertIn(68, result["resolved"])

        # Verify test executor was invoked
        mock_test_executor.assert_called_once_with("tests/test_upstream_reconcile.py", self.workspace_dir)

        # Verify label and comment were applied
        mock_provider.add_label.assert_called_once_with(68, "status:fixed-resolved")
        mock_provider.comment_issue.assert_called_once()
        comment_arg = mock_provider.comment_issue.call_args[0][1]
        self.assertIn("tests/test_upstream_reconcile.py", comment_arg)

    def test_reconcile_upstream_compiler_backlog_with_annotations_passing(self):
        """Verify passing test target in issue body annotation transitions compiler issue to status:fixed-resolved."""
        mock_provider = MagicMock()
        mock_provider.list_issues.return_value = [
            {
                "number": 68,
                "title": "Clean Landing Zone Reconciler",
                "state": "OPEN",
                "labels": [{"name": "compiler"}],
                "body": "Implement hook.\n<!-- test-target: tests/test_upstream_reconcile.py -->\nDone.",
            }
        ]

        mock_test_executor = MagicMock(return_value=(True, "OK"))

        result = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            test_executor=mock_test_executor,
        )

        self.assertEqual(result["processed"], 1)
        self.assertEqual(result["passed"], 1)
        self.assertEqual(result["failed"], 0)
        self.assertIn(68, result["resolved"])
        mock_provider.add_label.assert_called_once_with(68, "status:fixed-resolved")

    def test_reconcile_upstream_compiler_backlog_failing_tests_not_resolved(self):
        """Verify failing test target leaves compiler issue OPEN and does NOT mark resolved."""
        mock_provider = MagicMock()
        mock_provider.list_issues.return_value = [
            {
                "number": 68,
                "title": "Clean Landing Zone Reconciler",
                "state": "OPEN",
                "labels": [{"name": "compiler"}],
                "body": "Test-Target: tests/test_failing.py",
            }
        ]

        mock_test_executor = MagicMock(return_value=(False, "FAIL: test_something (AssertionError)"))

        result = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            test_executor=mock_test_executor,
        )

        self.assertEqual(result["processed"], 1)
        self.assertEqual(result["passed"], 0)
        self.assertEqual(result["failed"], 1)
        self.assertEqual(result["resolved"], [])
        mock_provider.add_label.assert_not_called()
        mock_provider.comment_issue.assert_not_called()

    def test_reconcile_upstream_compiler_backlog_already_resolved_skipped(self):
        """Verify already resolved issue is not re-resolved or commented."""
        mock_provider = MagicMock()
        mock_provider.list_issues.return_value = [
            {
                "number": 68,
                "title": "Clean Landing Zone Reconciler",
                "state": "OPEN",
                "labels": [{"name": "status:fixed-resolved"}],
                "body": "Test-Target: tests/test_upstream_reconcile.py",
            }
        ]

        mock_test_executor = MagicMock(return_value=(True, "OK"))

        result = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            test_executor=mock_test_executor,
        )

        self.assertEqual(result["processed"], 1)
        self.assertEqual(result["skipped"], 1)
        self.assertEqual(result["resolved"], [])
        mock_provider.add_label.assert_not_called()
        mock_provider.comment_issue.assert_not_called()

    def test_reconcile_upstream_compiler_backlog_multiple_test_targets(self):
        """Verify all test targets must pass for resolution; partial pass fails."""
        mock_provider = MagicMock()
        mock_provider.list_issues.return_value = [
            {
                "number": 68,
                "title": "Clean Landing Zone Reconciler",
                "state": "OPEN",
                "labels": [],
                "body": "Test-Targets: tests/test_a.py, tests/test_b.py",
            }
        ]

        # Case 1: First passes, second fails
        def side_effect(target, cwd):
            if target == "tests/test_a.py":
                return (True, "OK")
            return (False, "FAILED")

        mock_test_executor = MagicMock(side_effect=side_effect)

        result = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            test_executor=mock_test_executor,
        )

        self.assertEqual(result["failed"], 1)
        self.assertEqual(result["resolved"], [])
        mock_provider.add_label.assert_not_called()

        # Case 2: Both pass
        mock_test_executor2 = MagicMock(return_value=(True, "OK"))
        result2 = reconcile_upstream_compiler_backlog(
            workspace_dir=self.workspace_dir,
            rules=self.github_rules,
            provider_adapter=mock_provider,
            test_executor=mock_test_executor2,
        )
        self.assertEqual(result2["passed"], 1)
        self.assertEqual(result2["resolved"], [68])
        mock_provider.add_label.assert_called_once_with(68, "status:fixed-resolved")


    def test_execute_test_target(self):
        """Verify execute_test_target executes with custom executor and subprocess fallback."""
        # 1. Custom callable
        def mock_exec(target, ws):
            return (True, "CUSTOM PASS")
        success, out = execute_test_target("tests/test_foo.py", self.workspace_dir, test_executor=mock_exec)
        self.assertTrue(success)
        self.assertEqual(out, "CUSTOM PASS")

        # 2. Subprocess mocked execution
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0, stdout="Ran 1 test\nOK", stderr="")
            success, out = execute_test_target("tests/test_foo.py", self.workspace_dir)
            self.assertTrue(success)
            self.assertIn("Ran 1 test", out)

        # 3. Subprocess failure
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=1, stdout="", stderr="FAIL: test_x")
            success, out = execute_test_target("tests/test_foo.py", self.workspace_dir)
            self.assertFalse(success)
            self.assertIn("FAIL", out)


if __name__ == "__main__":
    unittest.main()

