"""
Unit tests for mechanical commit message non-closure validation gate (Pillar 1).
Verifies scripts/verify_commit_messages.py and commit-msg hook enforcement (.pipeline/constitution.md:266).
Resolves Issue #315.
"""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
VERIFY_SCRIPT = REPO_ROOT / "scripts" / "verify_commit_messages.py"
SETUP_HOOKS_SCRIPT = REPO_ROOT / "scripts" / "setup_git_hooks.py"

# Add scripts directory to sys.path for direct module import testing
sys.path.insert(0, str(REPO_ROOT / "scripts"))
from verify_commit_messages import verify_text, format_violation_error, main as verify_main


class TestCommitMessageValidatorLogic(unittest.TestCase):
    """Direct unit tests for commit message regex parsing and error formatting."""

    def test_valid_neutral_commit_messages(self):
        """Neutral citations and regular commit messages must produce zero violation matches."""
        valid_messages = [
            "feat(core): update model (refs #123)",
            "fix(parser): syntax (#456)",
            "docs: update handoff (#12)",
            "feat(compiler): multi-mode FMECA extraction (#50, #51)",
            "chore: routine housekeeping without issue reference",
            "fix: correct edge condition in statechart parser",
            "refactor: clean up signal matrix refs #99",
            "feat(gui): implement telemetry panel (ref #10)",
            "fix(validator): resolve tokenizer blind spot (refs #287)",
        ]
        for msg in valid_messages:
            matches = verify_text(msg)
            self.assertEqual(matches, [], f"False positive match on valid message: {msg}")

    def test_forbidden_auto_closing_commit_messages(self):
        """Auto-closing trigger keywords preceding #<id> must be detected."""
        forbidden_messages = [
            ("fixes #123", ["fixes #123"]),
            ("closes #45", ["closes #45"]),
            ("resolved #789", ["resolved #789"]),
            ("Fix #1", ["Fix #1"]),
            ("feat(conops): operational authority (fixes #313)", ["fixes #313"]),
            ("fix(conops): remediate compiler engine (fixes #296, #297)", ["fixes #296"]),
            ("docs: update handoff (resolved #12)", ["resolved #12"]),
            ("feat: add feature closes #99", ["closes #99"]),
            ("fix: #100", ["fix: #100"]),
            ("fixes: #101", ["fixes: #101"]),
            ("closes: #102", ["closes: #102"]),
            ("resolves: #103", ["resolves: #103"]),
        ]
        for msg, expected_triggers in forbidden_messages:
            matches = verify_text(msg)
            self.assertTrue(len(matches) > 0, f"Failed to detect forbidden trigger in: {msg}")
            for trig in expected_triggers:
                self.assertIn(trig, matches)

    def test_format_violation_error_cites_constitution(self):
        """Violation error string must cite .pipeline/constitution.md:266 and specify required format."""
        msg = "feat: some feature (fixes #123)"
        matches = verify_text(msg)
        err = format_violation_error(matches, context="test context", commit_msg=msg)
        self.assertIn(".pipeline/constitution.md:266", err)
        self.assertIn("(refs #<id>)", err)
        self.assertIn("fixes #123", err)
        self.assertIn("Offending message:", err)


class TestCommitMessageCLIExecution(unittest.TestCase):
    """Tests verify_commit_messages.py command-line execution and argument handling."""

    def test_check_text_valid_returns_zero(self):
        res = subprocess.run(
            [sys.executable, str(VERIFY_SCRIPT), "--check-text", "feat(core): update model (refs #123)"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"Expected returncode 0 for valid text, got {res.returncode}\n{res.stderr}")
        self.assertEqual(res.stderr, "")

    def test_check_text_invalid_returns_one_with_diagnostic(self):
        res = subprocess.run(
            [sys.executable, str(VERIFY_SCRIPT), "--check-text", "fix(parser): resolve syntax error (fixes #456)"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 1, f"Expected returncode 1 for invalid text, got {res.returncode}")
        self.assertIn(".pipeline/constitution.md:266", res.stderr)
        self.assertIn("fixes #456", res.stderr)
        self.assertIn("(refs #<id>)", res.stderr)

    def test_msg_file_valid_and_invalid(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            valid_file = Path(tmpdir) / "valid_msg.txt"
            valid_file.write_text("feat(core): clean commit (refs #99)\n", encoding="utf-8")

            invalid_file = Path(tmpdir) / "invalid_msg.txt"
            invalid_file.write_text("fix(core): bad commit (closes #99)\n", encoding="utf-8")

            # Valid file
            res_valid = subprocess.run(
                [sys.executable, str(VERIFY_SCRIPT), "--msg-file", str(valid_file)],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res_valid.returncode, 0)

            # Invalid file
            res_invalid = subprocess.run(
                [sys.executable, str(VERIFY_SCRIPT), "--msg-file", str(invalid_file)],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res_invalid.returncode, 1)
            self.assertIn(".pipeline/constitution.md:266", res_invalid.stderr)

    def test_missing_msg_file_returns_one(self):
        res = subprocess.run(
            [sys.executable, str(VERIFY_SCRIPT), "--msg-file", "/nonexistent/path/msg.txt"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("does not exist", res.stderr)

    def test_no_arguments_exits_code_two(self):
        res = subprocess.run(
            [sys.executable, str(VERIFY_SCRIPT)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 2)


class TestCommitMessageGitRepositoryIntegration(unittest.TestCase):
    """Hermetic tests using isolated temporary Git repositories."""

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp_dir.name) / "repo"
        self.repo.mkdir()

        # Initialize git repo
        subprocess.run(["git", "init"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test Runner"], cwd=self.repo, check=True, capture_output=True)

        # Copy scripts
        scripts_dir = self.repo / "scripts"
        scripts_dir.mkdir()
        (scripts_dir / "verify_commit_messages.py").write_text(VERIFY_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        (scripts_dir / "verify_commit_messages.py").chmod(0o755)
        (scripts_dir / "setup_git_hooks.py").write_text(SETUP_HOOKS_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        (scripts_dir / "setup_git_hooks.py").chmod(0o755)

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_head_and_range_validation_in_git_repo(self):
        verify_script = self.repo / "scripts" / "verify_commit_messages.py"
        test_file = self.repo / "test.txt"

        # Commit 1: Valid neutral commit
        test_file.write_text("line 1\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "feat: initial commit (#1)"], cwd=self.repo, check=True, capture_output=True)

        # Commit 2: Another valid neutral commit
        test_file.write_text("line 2\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "feat(core): update pipeline (refs #10)"], cwd=self.repo, check=True, capture_output=True)

        # Verify HEAD passes
        res_head = subprocess.run(
            [sys.executable, str(verify_script), "--head"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_head.returncode, 0, f"Expected HEAD check to pass, got:\n{res_head.stderr}")

        # Commit 3: Invalid commit with auto-closing trigger
        test_file.write_text("line 3\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "fix(bug): resolve issue (fixes #20)"], cwd=self.repo, check=True, capture_output=True)

        # Verify HEAD now fails
        res_head2 = subprocess.run(
            [sys.executable, str(verify_script), "--head"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_head2.returncode, 1, "Expected HEAD check to fail on 'fixes #20'")
        self.assertIn(".pipeline/constitution.md:266", res_head2.stderr)
        self.assertIn("fixes #20", res_head2.stderr)

        # Verify range validation: HEAD~1..HEAD fails, but HEAD~2..HEAD~1 passes
        res_range_bad = subprocess.run(
            [sys.executable, str(verify_script), "--range", "HEAD~1..HEAD"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_range_bad.returncode, 1)

        res_range_good = subprocess.run(
            [sys.executable, str(verify_script), "--range", "HEAD~2..HEAD~1"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_range_good.returncode, 0, f"Expected range to pass, got:\n{res_range_good.stderr}")

    def test_git_commit_msg_hook_blocks_auto_closing_commits(self):
        """Installing the commit-msg hook must cause 'git commit' to mechanically fail on forbidden keywords."""
        setup_script = self.repo / "scripts" / "setup_git_hooks.py"

        # Create dummy pipeline dirs to satisfy setup_git_hooks
        for d in [".pipeline", "skills", "rules", ".agents", "docs"]:
            (self.repo / d).mkdir(parents=True, exist_ok=True)
            (self.repo / d / "dummy.txt").write_text("dummy", encoding="utf-8")

        # Mock scripts/verify_subagent_output.py to always succeed so pre-commit passes cleanly
        (self.repo / "scripts" / "verify_subagent_output.py").write_text(
            "#!/usr/bin/env python3\nimport sys\nsys.exit(0)\n",
            encoding="utf-8",
        )
        (self.repo / "scripts" / "verify_subagent_output.py").chmod(0o755)

        # Initial commit before hooks
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=self.repo, check=True, capture_output=True)

        # Install hooks
        res_setup = subprocess.run(
            [sys.executable, str(setup_script), "--install"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_setup.returncode, 0)
        hook_path = self.repo / ".git" / "hooks" / "commit-msg"
        self.assertTrue(hook_path.exists())
        self.assertTrue(os.access(str(hook_path), os.X_OK))

        # Attempt commit with forbidden auto-closing keyword (must fail via commit-msg hook)
        dummy_file = self.repo / "dummy_edit.txt"
        dummy_file.write_text("update 1", encoding="utf-8")
        subprocess.run(["git", "add", "dummy_edit.txt"], cwd=self.repo, check=True, capture_output=True)

        commit_bad = subprocess.run(
            ["git", "commit", "-m", "fix(telemetry): update sensor readings (fixes #42)"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(commit_bad.returncode, 0, "Git commit should have been rejected by commit-msg hook")
        combined_output = commit_bad.stdout + commit_bad.stderr
        self.assertIn(".pipeline/constitution.md:266", combined_output)
        self.assertIn("fixes #42", combined_output)

        # Attempt commit with valid neutral citation (must succeed)
        commit_good = subprocess.run(
            ["git", "commit", "-m", "fix(telemetry): update sensor readings (refs #42)"],
            cwd=self.repo,
            capture_output=True,
            text=True,
        )
        self.assertEqual(
            commit_good.returncode,
            0,
            f"Git commit with neutral citation should have succeeded.\nOutput: {commit_good.stdout}\nStderr: {commit_good.stderr}",
        )


class TestRepositoryHeadStatus(unittest.TestCase):
    """Tests execution of --head in active workspace."""

    def test_run_head_on_current_workspace(self):
        res = subprocess.run(
            [sys.executable, str(VERIFY_SCRIPT), "--head"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
        # We assert that the command executes cleanly and returns an expected integer code (0 or 1)
        self.assertIn(res.returncode, (0, 1))
        if res.returncode == 1:
            self.assertIn(".pipeline/constitution.md:266", res.stderr)


if __name__ == "__main__":
    unittest.main()
