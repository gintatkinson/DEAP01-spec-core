"""Unit tests for skills/spec-orchestrator/scripts/create_issue.sh.

Tests parameter validation and usage message reporting when insufficient
arguments are supplied.
"""

import os
import subprocess
import unittest

SCRIPT_PATH = os.path.abspath(
    os.path.join(
        os.path.dirname(__file__),
        "..",
        "skills",
        "spec-orchestrator",
        "scripts",
        "create_issue.sh",
    )
)


class TestCreateIssueScript(unittest.TestCase):
    """Test suite verifying argument validation and usage output of create_issue.sh."""

    def test_script_exists_and_is_executable(self) -> None:
        """Verify that create_issue.sh exists on disk."""
        self.assertTrue(os.path.exists(SCRIPT_PATH), f"Script not found at {SCRIPT_PATH}")

    def test_zero_arguments_prints_usage(self) -> None:
        """Running with zero arguments must exit non-zero, print usage to stderr, and not crash with unbound variable."""
        result = subprocess.run(
            ["bash", SCRIPT_PATH],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Usage:", result.stderr)
        self.assertIn("<body-file> <label> <title> [repo]", result.stderr)
        self.assertNotIn("unbound variable", result.stderr)

    def test_one_argument_prints_usage(self) -> None:
        """Running with 1 argument must exit non-zero, print usage to stderr, and not crash with unbound variable."""
        result = subprocess.run(
            ["bash", SCRIPT_PATH, "some_file.md"],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Usage:", result.stderr)
        self.assertIn("<body-file> <label> <title> [repo]", result.stderr)
        self.assertNotIn("unbound variable", result.stderr)

    def test_two_arguments_prints_usage(self) -> None:
        """Running with 2 arguments must exit non-zero, print usage to stderr, and not crash with unbound variable."""
        result = subprocess.run(
            ["bash", SCRIPT_PATH, "some_file.md", "feature"],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Usage:", result.stderr)
        self.assertIn("<body-file> <label> <title> [repo]", result.stderr)
        self.assertNotIn("unbound variable", result.stderr)


if __name__ == "__main__":
    unittest.main()
