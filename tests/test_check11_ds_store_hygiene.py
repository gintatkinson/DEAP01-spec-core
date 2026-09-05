import io
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from scripts.verify_downstream_baseline import check_no_ds_store_files
from parity_auditor.core.workspace import WorkspaceRepository


class TestCheck11DSStoreHygiene(unittest.TestCase):
    """Test suite for Check 11: .DS_Store hygiene, auto-clean, and git index verification."""

    def _init_git_repo(self, path: str) -> None:
        """Initialize a local git repository in path for tracking tests."""
        subprocess.run(["git", "init"], cwd=path, check=True, capture_output=True)
        subprocess.run(
            ["git", "config", "user.name", "Test User"],
            cwd=path,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=path,
            check=True,
            capture_output=True,
        )

    def test_zero_ds_store_files_passes(self):
        """Verify Check 11 passes cleanly when zero .DS_Store files exist."""
        with tempfile.TemporaryDirectory() as tmpdir:
            self._init_git_repo(tmpdir)
            stdout_buf = io.StringIO()
            with redirect_stdout(stdout_buf):
                check_no_ds_store_files(tmpdir)

            output = stdout_buf.getvalue()
            self.assertIn("Success: Check 11 verified (zero .DS_Store files found).", output)

    def test_transient_untracked_ds_store_files_autocleaned_and_passes(self):
        """Verify Check 11 automatically deletes transient untracked .DS_Store files and passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            self._init_git_repo(tmpdir)

            # Create transient .DS_Store files
            root_ds = os.path.join(tmpdir, ".DS_Store")
            sub_dir = os.path.join(tmpdir, "docs", "epics")
            os.makedirs(sub_dir, exist_ok=True)
            sub_ds = os.path.join(sub_dir, ".DS_Store")

            with open(root_ds, "wb") as f:
                f.write(b"Mac OS X Finder Metadata")
            with open(sub_ds, "wb") as f:
                f.write(b"Mac OS X Finder Metadata")

            self.assertTrue(os.path.exists(root_ds))
            self.assertTrue(os.path.exists(sub_ds))

            stdout_buf = io.StringIO()
            with redirect_stdout(stdout_buf):
                check_no_ds_store_files(tmpdir)

            output = stdout_buf.getvalue()
            self.assertIn("Notice: [Cleaned] Removed 2 transient untracked .DS_Store file(s):", output)
            self.assertIn(".DS_Store", output)
            self.assertIn(os.path.join("docs", "epics", ".DS_Store"), output)
            self.assertIn(
                "Success: Check 11 verified (zero tracked .DS_Store files, transient files cleaned).",
                output,
            )

            # Assert files were deleted from disk
            self.assertFalse(os.path.exists(root_ds))
            self.assertFalse(os.path.exists(sub_ds))

    def test_tracked_ds_store_files_fails_with_exit_code_1(self):
        """Verify Check 11 fails closed with exit code 1 when .DS_Store is tracked in git index."""
        with tempfile.TemporaryDirectory() as tmpdir:
            self._init_git_repo(tmpdir)

            root_ds = os.path.join(tmpdir, ".DS_Store")
            with open(root_ds, "wb") as f:
                f.write(b"Tracked metadata")

            # Force add .DS_Store to git index
            subprocess.run(["git", "add", "-f", ".DS_Store"], cwd=tmpdir, check=True, capture_output=True)

            stderr_buf = io.StringIO()
            stdout_buf = io.StringIO()
            with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
                with self.assertRaises(SystemExit) as cm:
                    check_no_ds_store_files(tmpdir)

            self.assertEqual(cm.exception.code, 1)
            err_output = stderr_buf.getvalue()
            self.assertIn("ERROR: Check 11 failed: Found 1 tracked/committed .DS_Store file(s) in git index: .DS_Store", err_output)
            # Tracked file remains on disk
            self.assertTrue(os.path.exists(root_ds))

    def test_mixed_tracked_and_untracked_ds_store_files(self):
        """Verify Check 11 removes untracked .DS_Store and fails on tracked .DS_Store."""
        with tempfile.TemporaryDirectory() as tmpdir:
            self._init_git_repo(tmpdir)

            tracked_ds = os.path.join(tmpdir, ".DS_Store")
            sub_dir = os.path.join(tmpdir, "schema")
            os.makedirs(sub_dir, exist_ok=True)
            untracked_ds = os.path.join(sub_dir, ".DS_Store")

            with open(tracked_ds, "wb") as f:
                f.write(b"Tracked")
            with open(untracked_ds, "wb") as f:
                f.write(b"Untracked")

            subprocess.run(["git", "add", "-f", ".DS_Store"], cwd=tmpdir, check=True, capture_output=True)

            stderr_buf = io.StringIO()
            stdout_buf = io.StringIO()
            with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
                with self.assertRaises(SystemExit) as cm:
                    check_no_ds_store_files(tmpdir)

            self.assertEqual(cm.exception.code, 1)
            self.assertIn("Notice: [Cleaned] Removed 1 transient untracked .DS_Store file(s):", stdout_buf.getvalue())
            self.assertIn("ERROR: Check 11 failed: Found 1 tracked/committed .DS_Store file(s) in git index: .DS_Store", stderr_buf.getvalue())

            # Untracked was removed, tracked was not removed
            self.assertFalse(os.path.exists(untracked_ds))
            self.assertTrue(os.path.exists(tracked_ds))

    def test_workspace_repository_get_markdown_files_and_features_ignores_ds_store(self):
        """Verify WorkspaceRepository ignores .DS_Store and non-markdown/hidden files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = WorkspaceRepository(tmpdir)

            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(docs_dir, exist_ok=True)

            valid_md = os.path.join(docs_dir, "feat-01-control.md")
            with open(valid_md, "w", encoding="utf-8") as f:
                f.write("---\ntitle: Control\nlabels: [feature]\n---\n# Feat 1\n")

            ds_file = os.path.join(docs_dir, ".DS_Store")
            with open(ds_file, "wb") as f:
                f.write(b"DS_Store bytes")

            hidden_md = os.path.join(docs_dir, ".hidden.md")
            with open(hidden_md, "w", encoding="utf-8") as f:
                f.write("# Hidden\n")

            txt_file = os.path.join(docs_dir, "notes.txt")
            with open(txt_file, "w", encoding="utf-8") as f:
                f.write("text notes\n")

            # Test get_markdown_files
            md_files = repo.get_markdown_files(docs_dir)
            self.assertEqual(md_files, [valid_md])

            # Test get_feature_files
            feature_files = repo.get_feature_files(docs_dir)
            self.assertEqual(len(feature_files), 1)
            self.assertEqual(feature_files[0].filename, "feat-01-control.md")


if __name__ == "__main__":
    unittest.main()
