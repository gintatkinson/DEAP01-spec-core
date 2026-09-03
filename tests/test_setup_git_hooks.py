"""
Tests for scripts/setup_git_hooks.py CLI behavior and safety.
Verifies setup_git_hooks.py default execution and --install flag support (Issue #171).
"""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPT_PATH = REPO_ROOT / "scripts" / "setup_git_hooks.py"


class TestSetupGitHooksCLI(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp_dir.name) / "repo"
        self.repo.mkdir()

        # Initialize git repo
        subprocess.run(["git", "init"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test Runner"], cwd=self.repo, check=True, capture_output=True)

        # Create dummy pipeline infrastructure directories and files
        for d in [".pipeline", "skills", "rules", "scripts", ".agents"]:
            dir_path = self.repo / d
            dir_path.mkdir(parents=True, exist_ok=True)
            (dir_path / "dummy.txt").write_text("content", encoding="utf-8")

        # Initial commit so HEAD exists and staging works
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=self.repo, check=True, capture_output=True)

        # Copy script into the temporary repo scripts directory
        script_dest = self.repo / "scripts" / "setup_git_hooks.py"
        script_dest.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
        script_dest.chmod(0o755)

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_no_args_installs_hooks_and_modifies_gitignore(self):
        """Calling setup_git_hooks.py without arguments must install hooks and whitelist infrastructure by default."""
        script_file = self.repo / "scripts" / "setup_git_hooks.py"
        gitignore_file = self.repo / ".gitignore"
        pre_commit_file = self.repo / ".git" / "hooks" / "pre-commit"

        res = subprocess.run(
            [sys.executable, str(script_file)],
            cwd=self.repo,
            capture_output=True,
            text=True,
            timeout=10,
        )

        self.assertEqual(res.returncode, 0, f"Expected returncode 0 without arguments, got {res.returncode}\nSTDERR:\n{res.stderr}")
        self.assertTrue(pre_commit_file.exists(), "pre-commit hook was not created")
        self.assertTrue(os.access(str(pre_commit_file), os.X_OK), "pre-commit hook is not executable")
        self.assertTrue(gitignore_file.exists(), ".gitignore was not created")
        gitignore_content = gitignore_file.read_text(encoding="utf-8")
        self.assertIn("Pipeline infrastructure", gitignore_content)

    def test_install_flag_installs_hooks_and_modifies_gitignore(self):
        """Calling setup_git_hooks.py with --install must install hooks and whitelist infrastructure."""
        script_file = self.repo / "scripts" / "setup_git_hooks.py"
        gitignore_file = self.repo / ".gitignore"
        pre_commit_file = self.repo / ".git" / "hooks" / "pre-commit"

        res = subprocess.run(
            [sys.executable, str(script_file), "--install"],
            cwd=self.repo,
            capture_output=True,
            text=True,
            timeout=10,
        )

        self.assertEqual(res.returncode, 0, f"Expected returncode 0 with --install, got {res.returncode}\nSTDERR:\n{res.stderr}")
        self.assertTrue(pre_commit_file.exists(), "pre-commit hook was not created")
        self.assertTrue(os.access(str(pre_commit_file), os.X_OK), "pre-commit hook is not executable")
        self.assertTrue(gitignore_file.exists(), ".gitignore was not created")
        gitignore_content = gitignore_file.read_text(encoding="utf-8")
        self.assertIn("Pipeline infrastructure", gitignore_content)

    def test_help_prints_usage_and_exits_zero(self):
        """Calling setup_git_hooks.py with --help must print usage and exit 0 without side effects."""
        script_file = self.repo / "scripts" / "setup_git_hooks.py"
        pre_commit_file = self.repo / ".git" / "hooks" / "pre-commit"

        res = subprocess.run(
            [sys.executable, str(script_file), "--help"],
            cwd=self.repo,
            capture_output=True,
            text=True,
            timeout=10,
        )

        self.assertEqual(res.returncode, 0, f"Expected 0 exit code with --help, got {res.returncode}")
        self.assertTrue("usage:" in res.stdout.lower() or "usage:" in res.stderr.lower())
        self.assertTrue("--install" in res.stdout or "--install" in res.stderr)
        self.assertFalse(pre_commit_file.exists(), "pre-commit hook should not have been created on --help")

    def test_invalid_flag_exits_code_2(self):
        """Calling setup_git_hooks.py with an invalid flag must exit with returncode 2."""
        script_file = self.repo / "scripts" / "setup_git_hooks.py"

        res = subprocess.run(
            [sys.executable, str(script_file), "--invalid-flag"],
            cwd=self.repo,
            capture_output=True,
            text=True,
            timeout=10,
        )

        self.assertEqual(res.returncode, 2, f"Expected returncode 2 for invalid flag, got {res.returncode}")
        self.assertTrue("unrecognized arguments" in res.stderr.lower() or "usage:" in res.stderr.lower())


if __name__ == "__main__":
    unittest.main()

