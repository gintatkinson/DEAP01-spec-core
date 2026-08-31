#!/usr/bin/env python3
"""
Unit tests for Subagent Prompt Payload Linter (`scripts/lint_subagent_prompt.py`).
"""

import os
import sys
import tempfile
import subprocess
import unittest

# Ensure repo root and scripts are on sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from scripts.lint_subagent_prompt import lint_subagent_prompt


class TestSubagentPromptLinter(unittest.TestCase):
    """Test suite for subagent prompt linting rules and CLI."""

    def setUp(self):
        self.valid_prompt = """
You are a context-isolated Feature Implementation Worker for DEAP-spec-core.

Task: Implement FEAT-01 (Flight Guidance Computer).

Instructions:
1. Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before performing any actions.
2. Implement the logical operations and unit tests for FEAT-01.
3. If defects or anomalies are detected, record them using `gh issue create` (GitHub) or `glab issue create` (GitLab).
4. Run validation checks.

PROCEED
"""

    def test_valid_prompt_passes(self):
        """Verify that a compliant prompt payload passes with zero lint errors."""
        errors = lint_subagent_prompt(self.valid_prompt)
        self.assertEqual(errors, [])

    def test_missing_view_file_skill_md_directive(self):
        """Verify that missing view_file directive on SKILL.md as step 1 is flagged."""
        prompt = """
Task: Implement FEAT-01.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("view_file" in e and "SKILL.md" in e for e in errors))

    def test_view_file_without_step1_or_prerequisite_directive(self):
        """Verify that mentioning view_file and SKILL.md without sequencing directive is flagged."""
        prompt = """
Task: Implement FEAT-01.
You may refer to SKILL.md using view_file at any point.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("view_file" in e and "SKILL.md" in e for e in errors))

    def test_multiple_features_scope_violation(self):
        """Verify that prompts specifying multiple features violate micro-task scope."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01 and FEAT-02.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("multiple Features" in e for e in errors))

    def test_multiple_user_stories_scope_violation(self):
        """Verify that prompts specifying multiple user stories violate micro-task scope."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement US-01 and US-02.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("multiple User Stories" in e for e in errors))

    def test_multiple_epics_scope_violation(self):
        """Verify that prompts specifying multiple epics violate micro-task scope."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement EPIC-01 and EPIC-02.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("multiple Epics" in e for e in errors))

    def test_multiple_use_cases_scope_violation(self):
        """Verify that prompts specifying multiple use cases violate micro-task scope."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement UC-01 and UC-02.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("multiple Use Cases" in e for e in errors))

    def test_batch_phrasing_scope_violation(self):
        """Verify that batch processing keywords violate micro-task scope."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Process all features in batch execution mode.
If defects found, use gh issue create or glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("violates micro-task scope" in e for e in errors))

    def test_missing_glab_issue_create(self):
        """Verify that missing glab issue create directive is flagged."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01.
If defects found, use gh issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("glab issue create" in e for e in errors))

    def test_missing_gh_issue_create(self):
        """Verify that missing gh issue create directive is flagged."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01.
If defects found, use glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("gh issue create" in e for e in errors))

    def test_missing_proceed_token(self):
        """Verify that missing PROCEED authorization token is flagged."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01.
If defects found, use gh issue create and glab issue create.
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("PROCEED" in e for e in errors))

    def test_forbidden_issue_close(self):
        """Verify that gh/glab issue close directives are rejected."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01.
If defects found, use gh issue create or glab issue create.
Once complete, run gh issue close 123.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("gh/glab issue close" in e for e in errors))

    def test_truncation_indicator(self):
        """Verify that ellipsis or summarization markers are flagged."""
        prompt = """
Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running.
Task: Implement FEAT-01 [...]
If defects found, use gh issue create and glab issue create.
PROCEED
"""
        errors = lint_subagent_prompt(prompt)
        self.assertTrue(any("truncation" in e for e in errors))

    def test_empty_prompt_payload(self):
        """Verify that empty string or whitespace prompt is rejected."""
        self.assertTrue(len(lint_subagent_prompt("")) > 0)
        self.assertTrue(len(lint_subagent_prompt("   \n\t  ")) > 0)

    def test_cli_execution_pass(self):
        """Verify CLI interface returns 0 on valid prompt file."""
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as tf:
            tf.write(self.valid_prompt)
            temp_path = tf.name

        try:
            cmd = [sys.executable, os.path.join(PROJECT_ROOT, "scripts", "lint_subagent_prompt.py"), temp_path]
            res = subprocess.run(cmd, capture_output=True, text=True)
            self.assertEqual(res.returncode, 0, f"Expected CLI 0, got {res.returncode}. Output: {res.stdout}\n{res.stderr}")
            self.assertIn("PASSED", res.stdout)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_cli_execution_fail(self):
        """Verify CLI interface returns non-zero on invalid prompt string."""
        cmd = [sys.executable, os.path.join(PROJECT_ROOT, "scripts", "lint_subagent_prompt.py"), "Invalid prompt without required directives"]
        res = subprocess.run(cmd, capture_output=True, text=True)
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("FAILED", res.stderr)


if __name__ == "__main__":
    unittest.main()
