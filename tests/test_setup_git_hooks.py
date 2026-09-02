"""
Tests for scripts/setup_git_hooks.py CLI behavior and safety.
Verifies CORE #84: setup_git_hooks.py requires explicit --install verb.
"""

import os
import subprocess
import sys
from pathlib import Path
import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPT_PATH = REPO_ROOT / "scripts" / "setup_git_hooks.py"


@pytest.fixture
def temp_git_repo(tmp_path):
    """Create a temporary git repository with standard pipeline directories."""
    repo = tmp_path / "repo"
    repo.mkdir()
    # Initialize git repo
    subprocess.run(["git", "init"], cwd=repo, check=True, capture_output=True)
    subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo, check=True, capture_output=True)
    subprocess.run(["git", "config", "user.name", "Test Runner"], cwd=repo, check=True, capture_output=True)

    # Create dummy pipeline infrastructure directories and files
    for d in [".pipeline", "skills", "rules", "scripts", ".agents"]:
        dir_path = repo / d
        dir_path.mkdir(parents=True, exist_ok=True)
        (dir_path / "dummy.txt").write_text("content", encoding="utf-8")

    # Initial commit so HEAD exists and staging works
    subprocess.run(["git", "add", "."], cwd=repo, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=repo, check=True, capture_output=True)

    # Copy script into the temporary repo scripts directory
    script_dest = repo / "scripts" / "setup_git_hooks.py"
    script_dest.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
    script_dest.chmod(0o755)

    return repo


def test_no_args_prints_usage_and_has_no_side_effects(temp_git_repo):
    """Calling setup_git_hooks.py without --install must exit non-zero, print usage, and produce no side effects."""
    script_file = temp_git_repo / "scripts" / "setup_git_hooks.py"
    gitignore_file = temp_git_repo / ".gitignore"
    hooks_dir = temp_git_repo / ".git" / "hooks"
    pre_commit_file = hooks_dir / "pre-commit"

    initial_gitignore_exists = gitignore_file.exists()

    res = subprocess.run(
        [sys.executable, str(script_file)],
        cwd=temp_git_repo,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert res.returncode != 0, f"Expected non-zero exit code when called without --install, got {res.returncode}"
    combined_output = (res.stdout + res.stderr).lower()
    assert "usage:" in combined_output, f"Expected 'usage:' in output, got:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}"

    # Verify no side effects
    assert not pre_commit_file.exists(), "pre-commit hook should not have been created"
    if not initial_gitignore_exists:
        assert not gitignore_file.exists() or "Pipeline infrastructure" not in gitignore_file.read_text(encoding="utf-8")


def test_help_prints_usage_and_exits_zero(temp_git_repo):
    """Calling setup_git_hooks.py with --help must print usage and exit 0 without side effects."""
    script_file = temp_git_repo / "scripts" / "setup_git_hooks.py"
    pre_commit_file = temp_git_repo / ".git" / "hooks" / "pre-commit"

    res = subprocess.run(
        [sys.executable, str(script_file), "--help"],
        cwd=temp_git_repo,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert res.returncode == 0, f"Expected 0 exit code with --help, got {res.returncode}"
    assert "usage:" in res.stdout.lower() or "usage:" in res.stderr.lower()
    assert "--install" in res.stdout or "--install" in res.stderr
    assert not pre_commit_file.exists(), "pre-commit hook should not have been created on --help"


def test_invalid_flag_exits_code_2(temp_git_repo):
    """Calling setup_git_hooks.py with an invalid flag must exit with returncode 2."""
    script_file = temp_git_repo / "scripts" / "setup_git_hooks.py"

    res = subprocess.run(
        [sys.executable, str(script_file), "--invalid-flag"],
        cwd=temp_git_repo,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert res.returncode == 2, f"Expected returncode 2 for invalid flag, got {res.returncode}"
    assert "unrecognized arguments" in res.stderr.lower() or "usage:" in res.stderr.lower()


def test_install_flag_installs_hooks_and_modifies_gitignore(temp_git_repo):
    """Calling setup_git_hooks.py with --install must install hooks and whitelist infrastructure."""
    script_file = temp_git_repo / "scripts" / "setup_git_hooks.py"
    gitignore_file = temp_git_repo / ".gitignore"
    pre_commit_file = temp_git_repo / ".git" / "hooks" / "pre-commit"

    res = subprocess.run(
        [sys.executable, str(script_file), "--install"],
        cwd=temp_git_repo,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert res.returncode == 0, f"Expected returncode 0 with --install, got {res.returncode}\nSTDERR:\n{res.stderr}"
    assert pre_commit_file.exists(), "pre-commit hook was not created"
    assert os.access(str(pre_commit_file), os.X_OK), "pre-commit hook is not executable"
    assert gitignore_file.exists(), ".gitignore was not created"
    gitignore_content = gitignore_file.read_text(encoding="utf-8")
    assert "Pipeline infrastructure" in gitignore_content
