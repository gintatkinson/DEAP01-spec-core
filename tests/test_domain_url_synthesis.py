#!/usr/bin/env python3
"""
Unit and Regression Test Suite for Domain Template URL Synthesis and CLI Parameter Support.

Tests Issue #363 / scripts/install_pipeline.sh:
1. Help text documentation:
   - Verifies that --help documents --domain-url and --domain-name flags.
2. CLI argument validation:
   - Verifies that omitting argument for --domain-url or --domain-name fails with exit code 1
     and emits clear error message to stderr.
   - Verifies that passing another flag as argument (e.g. --domain-url --provider gitlab)
     fails with exit code 1 and emits clear error message to stderr.
3. Provider decoupling & default GitHub domain template URL synthesis:
   - Verifies that running with --provider gitlab --domain-name "DEAP-uas-infrastructure-safety"
     synthesizes a valid GitHub remote URL (https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git)
     for the downstream customer onboarding command in README.md.
   - Asserts that no non-existent 'gitlab.com' domain URL is generated.
4. Explicit --domain-url flag support:
   - Verifies that --domain-url <URL> overrides default synthesis and embeds the explicit URL.
5. Equals syntax support:
   - Verifies that --domain-url=<URL> and --domain-name=<NAME> work as expected.
6. Git remote origin preservation:
   - Verifies that when a target directory already has a configured origin remote that is not
     DEAP01-spec-core, that remote URL is preserved in the generated README.md onboarding command.
7. Flag precedence:
   - Verifies that --domain-url takes precedence over --domain-name for clone URL synthesis.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis,
  DO-178C C/SPARK Ada code generation).

Pure Schema-Driven Invariant:
  Zero hardcoded domain concepts. All test runs operate in isolated temporary directories
  outside the workspace repository and leave zero residual state.
"""

import os
import re
import subprocess
import sys
import tempfile
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INSTALL_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "install_pipeline.sh")


class TestDomainUrlSynthesis(unittest.TestCase):
    """Regression test suite for domain template URL synthesis and CLI options in install_pipeline.sh."""

    def setUp(self):
        """Verify installer script exists and is executable."""
        self.assertTrue(os.path.isfile(INSTALL_SCRIPT), f"Installer not found at {INSTALL_SCRIPT}")
        self.assertTrue(os.access(INSTALL_SCRIPT, os.X_OK), f"Installer is not executable: {INSTALL_SCRIPT}")

    def _run_installer(self, *args, cwd=None):
        """Helper to invoke scripts/install_pipeline.sh with arguments."""
        cmd = ["bash", INSTALL_SCRIPT] + list(args)
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True
        )

    def test_help_shows_domain_options(self):
        """Runs `bash scripts/install_pipeline.sh --help` and asserts --domain-url and --domain-name are documented."""
        proc = self._run_installer("--help")
        self.assertEqual(proc.returncode, 0, f"Expected returncode 0, got {proc.returncode}. stderr: {proc.stderr}")
        self.assertIn("--domain-url", proc.stdout, "--domain-url flag must be documented in --help")
        self.assertIn("--domain-name", proc.stdout, "--domain-name flag must be documented in --help")

    def test_domain_url_missing_arg_fails(self):
        """Runs `bash scripts/install_pipeline.sh --domain-url` and asserts returncode == 1 with error on stderr."""
        # Case 1: --domain-url with no following argument
        proc1 = self._run_installer("--domain-url")
        self.assertEqual(proc1.returncode, 1, f"Expected returncode 1, got {proc1.returncode}")
        self.assertIn("Error: --domain-url requires a URL argument", proc1.stderr)

        # Case 2: --domain-url followed by another option flag
        proc2 = self._run_installer("--domain-url", "--provider", "gitlab")
        self.assertEqual(proc2.returncode, 1, f"Expected returncode 1, got {proc2.returncode}")
        self.assertIn("Error: --domain-url requires a URL argument", proc2.stderr)

    def test_domain_name_missing_arg_fails(self):
        """Runs `bash scripts/install_pipeline.sh --domain-name` and asserts returncode == 1 with error on stderr."""
        # Case 1: --domain-name with no following argument
        proc1 = self._run_installer("--domain-name")
        self.assertEqual(proc1.returncode, 1, f"Expected returncode 1, got {proc1.returncode}")
        self.assertIn("Error: --domain-name requires a name argument", proc1.stderr)

        # Case 2: --domain-name followed by another option flag
        proc2 = self._run_installer("--domain-name", "--provider", "gitlab")
        self.assertEqual(proc2.returncode, 1, f"Expected returncode 1, got {proc2.returncode}")
        self.assertIn("Error: --domain-name requires a name argument", proc2.stderr)

    def test_domain_remote_url_with_gitlab_provider_defaults_to_github(self):
        """Creates a temporary directory, runs installer with --provider gitlab --domain-name, asserts README output."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(
                temp_dir,
                "--provider", "gitlab",
                "--domain-name", "DEAP-uas-infrastructure-safety"
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                "git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command not found in README.md:\n{content}")

            # Verify that gitlab.com is NOT synthesized for the domain URL
            for line in content.splitlines():
                if "DEAP-uas-infrastructure-safety" in line:
                    self.assertNotIn(
                        "gitlab.com",
                        line,
                        f"Found forbidden 'gitlab.com' synthesized for domain URL in line: {line}"
                    )
                if "git clone" in line and ".tmp-pipeline" in line:
                    self.assertNotIn(
                        "gitlab.com",
                        line,
                        f"Found forbidden 'gitlab.com' in customer onboarding git clone command: {line}"
                    )

    def test_explicit_domain_url_flag(self):
        """Creates a temporary directory, runs installer with --domain-url, asserts README output."""
        custom_url = "https://example.com/custom/domain.git"
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(
                temp_dir,
                "--provider", "gitlab",
                "--domain-url", custom_url
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                f"git clone {custom_url} ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command not found in README.md:\n{content}")

    def test_explicit_domain_url_equals_syntax(self):
        """Creates a temporary directory, runs installer with --domain-url=..., asserts README output."""
        repo_url = "https://github.com/org/domain-repo.git"
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(
                temp_dir,
                f"--domain-url={repo_url}"
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                f"git clone {repo_url} ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command not found in README.md:\n{content}")

    def test_target_origin_remote_preserved(self):
        """Creates a temporary git repository with an origin remote, asserts that remote URL is preserved."""
        origin_url = "https://github.com/org/DEAP-uas-infrastructure-safety.git"
        with tempfile.TemporaryDirectory() as temp_dir:
            # Initialize git repository and set origin remote
            subprocess.run(["git", "init"], cwd=temp_dir, check=True, capture_output=True)
            subprocess.run(["git", "remote", "add", "origin", origin_url], cwd=temp_dir, check=True, capture_output=True)

            proc = self._run_installer(temp_dir)
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                f"git clone {origin_url} ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command with preserved origin not found:\n{content}")

    def test_explicit_domain_name_equals_syntax(self):
        """Creates a temporary directory, runs installer with --domain-name=..., asserts README output."""
        domain_name = "DEAP-custom-safety"
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(
                temp_dir,
                f"--domain-name={domain_name}"
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                f"git clone https://github.com/gintatkinson/{domain_name}.git ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command not found in README.md:\n{content}")

    def test_domain_url_precedence_over_domain_name(self):
        """Verifies that --domain-url takes precedence over --domain-name when both are specified."""
        explicit_url = "https://example.com/explicit-precedence.git"
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(
                temp_dir,
                "--domain-url", explicit_url,
                "--domain-name", "DEAP-ignored-name"
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated in target directory")

            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_clone_cmd = (
                f"git clone {explicit_url} ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content, f"Expected clone command with explicit URL not found:\n{content}")
            self.assertNotIn("DEAP-ignored-name.git", content, "Ignored domain name should not appear in clone URL")


if __name__ == "__main__":
    unittest.main()
