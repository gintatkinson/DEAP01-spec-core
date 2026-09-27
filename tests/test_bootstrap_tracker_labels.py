#!/usr/bin/env python3
"""
Unit tests for tracker label bootstrapping glab token resolution.

Verifies remediation of Issue #392:
- Rejects `glab auth token` output when it contains USAGE help banners, spaces, or newlines.
- Extracts token from `glab auth status --show-token` via keyring or standard token regex.
- Falls through cleanly when glab output is invalid.
"""

import os
import sys
import unittest
from unittest.mock import MagicMock, patch

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPTS_DIR = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "scripts")
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

from bootstrap_tracker_labels import GitLabV4LabelProvider, _resolve_token_from_git_credential


class TestGlabTokenResolution(unittest.TestCase):
    """Verifies sanitized glab token extraction in GitLabV4LabelProvider."""

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_token_usage_rejected(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies glab auth token returning USAGE banner is rejected and falls through."""
        mock_status = MagicMock()
        mock_status.returncode = 0
        mock_status.stdout = ""
        mock_status.stderr = ""

        mock_auth_token = MagicMock()
        mock_auth_token.returncode = 0
        mock_auth_token.stdout = (
            "\n"
            "  USAGE\n"
            "    glab auth <command> [command] [--flags]\n"
            "\n"
            "  COMMANDS\n"
            "    status    View authentication status.\n"
        )
        mock_auth_token.stderr = ""

        mock_run.side_effect = [mock_status, mock_auth_token]

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertIsNone(provider.token)
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_status_keyring(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies token extraction from glab auth status --show-token keyring output."""
        mock_status = MagicMock()
        mock_status.returncode = 0
        mock_status.stdout = (
            "gitlab.com\n"
            "  ✓ Logged in to gitlab.com as testuser (~/.config/glab-cli/config.yml)\n"
            "  ✓ Token found in operating system keyring: glpat-keyring-token-abc123\n"
        )
        mock_status.stderr = ""
        mock_run.return_value = mock_status

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertEqual(provider.token, "glpat-keyring-token-abc123")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_status_standard(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies token extraction from glab auth status --show-token standard format."""
        mock_status = MagicMock()
        mock_status.returncode = 0
        mock_status.stdout = "  ✓ Token: glpat-standard-token-xyz789\n"
        mock_status.stderr = ""
        mock_run.return_value = mock_status

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertEqual(provider.token, "glpat-standard-token-xyz789")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_status_stderr(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies token extraction when glab outputs auth status to stderr."""
        mock_status = MagicMock()
        mock_status.returncode = 0
        mock_status.stdout = ""
        mock_status.stderr = (
            "gitlab.com\n"
            "  ✓ Logged in to gitlab.com as testuser (~/.config/glab-cli/config.yml)\n"
            "  ✓ Token found in operating system keyring: glpat-keyring-token-stderr123\n"
        )
        mock_run.return_value = mock_status

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertEqual(provider.token, "glpat-keyring-token-stderr123")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_token_valid_fallback(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies clean token from glab auth token is accepted if auth status produced no token."""
        mock_status = MagicMock()
        mock_status.returncode = 1
        mock_status.stdout = ""
        mock_status.stderr = ""

        mock_auth_token = MagicMock()
        mock_auth_token.returncode = 0
        mock_auth_token.stdout = "glpat-valid-single-line-token\n"
        mock_auth_token.stderr = ""

        mock_run.side_effect = [mock_status, mock_auth_token]

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertEqual(provider.token, "glpat-valid-single-line-token")
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")

    @patch("shutil.which", return_value="/usr/local/bin/glab")
    @patch("netrc.netrc", side_effect=FileNotFoundError)
    @patch("bootstrap_tracker_labels._resolve_token_from_git_credential", return_value=None)
    @patch("subprocess.run")
    def test_resolve_token_glab_auth_token_with_spaces_rejected(
        self, mock_run, mock_cred, mock_netrc, mock_which
    ):
        """Verifies glab auth token returning text with spaces is rejected."""
        mock_status = MagicMock()
        mock_status.returncode = 1
        mock_status.stdout = ""
        mock_status.stderr = ""

        mock_auth_token = MagicMock()
        mock_auth_token.returncode = 0
        mock_auth_token.stdout = "not a valid token with spaces"
        mock_auth_token.stderr = ""

        mock_run.side_effect = [mock_status, mock_auth_token]

        with patch.dict(os.environ, {}, clear=True):
            provider = GitLabV4LabelProvider(
                server_url="https://gitlab.example.com",
                project_id="org/repo",
            )
            self.assertIsNone(provider.token)
            self.assertEqual(provider.token_type, "PRIVATE-TOKEN")


if __name__ == "__main__":
    unittest.main()
