#!/usr/bin/env python3
"""
Unit tests for Subagent Prompt Payload Linter (`scripts/lint_subagent_prompt.py`).
"""

import contextlib
import io
import os
import re
import sys
import tempfile
import subprocess
import unittest

# Ensure repo root and scripts are on sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from scripts.lint_subagent_prompt import (
    lint_subagent_prompt,
    check_repository_classification,
    validate_subagent_preflight,
)


class TestSubagentPromptLinter(unittest.TestCase):
    """Test suite for subagent prompt linting rules and CLI."""

    RULES_PATH = os.path.join(PROJECT_ROOT, "rules", "subagent-dispatch-standards.md")

    @staticmethod
    def _corpus_requirements():
        """Parse the six Requirement cells of the pre-flight checklist body verbatim."""
        with open(TestSubagentPromptLinter.RULES_PATH, "r", encoding="utf-8") as f:
            lines = f.read().splitlines()
        requirements = []
        for line in lines:
            if re.match(r"^\|\s*\d+\.\s", line):
                cells = [c.strip() for c in line.split("|")]
                if len(cells) > 3 and cells[2]:
                    requirements.append(cells[2])
        return requirements

    @staticmethod
    def _build_prompt(checklist_rows):
        checklist = "\n".join(f"- {row}" for row in checklist_rows)
        return f"""
You are a context-isolated Feature Implementation Worker for DEAP-spec-core.
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER

Task: Implement FEAT-01.

Normative Pre-Flight Checklist (verbatim from rules/subagent-dispatch-standards.md):
{checklist}

Instructions:
1. Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before performing any actions.
2. Implement the logical operations and unit tests for FEAT-01.
3. If defects or anomalies are detected, record them using `gh issue create` (GitHub) or `glab issue create` (GitLab).
4. Run validation checks.

PROCEED
"""

    def setUp(self):
        self.valid_prompt = self._build_prompt(self._corpus_requirements())

    def test_valid_prompt_passes(self):
        """Verify that a compliant prompt payload passes with zero lint errors."""
        errors = lint_subagent_prompt(self.valid_prompt)
        self.assertEqual(errors, [])

    def test_valid_prompt_with_non_compiler_classification(self):
        """Verify that prompts targeting non-compiler roles pass linting."""
        for classification in [
            "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE",
            "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE",
            "DOWNSTREAM_APPLICATION_WORKSPACE",
            "DOWNSTREAM_CUSTOMER_PROJECT",
        ]:
            prompt = self.valid_prompt.replace("UPSTREAM_SPEC_CORE_COMPILER", classification)
            errors = lint_subagent_prompt(prompt)
            self.assertEqual(errors, [], f"Classification {classification} had unexpected errors: {errors}")

    def test_repository_classification_acceptance(self):
        """Verify check_repository_classification accepts valid classification indicators across workspace types."""
        valid_samples = [
            "Repository Classification: UPSTREAM_SPEC_CORE_COMPILER",
            "Repository Classification: PARENT_DOMAIN_DISTRIBUTION_TEMPLATE",
            "Repository Classification: CHILD_DOMAIN_DISTRIBUTION_TEMPLATE",
            "Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT",
            "Repository Classification: DOWNSTREAM_APPLICATION_WORKSPACE",
            "Repository Classification: DOWNSTREAM_CUSTOMER_WORKSPACE",
            "Repository Classification: CUSTOMER_PROJECT_X",
            "**Repository Classification:** `DOWNSTREAM_CUSTOMER_PROJECT`",
            "**Classification**: `DOWNSTREAM_CUSTOMER_PROJECT`",
            "### Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT",
            "operating within classification `DOWNSTREAM_CUSTOMER_PROJECT`",
            "within classification 'PARENT_DOMAIN_DISTRIBUTION_TEMPLATE'",
            "Classification: DOWNSTREAM_CUSTOMER_PROJECT",
            "Classification = DOWNSTREAM_CUSTOMER_PROJECT",
            "Classification - DOWNSTREAM_CUSTOMER_PROJECT",
            "This prompt targets DOWNSTREAM_CUSTOMER_PROJECT workspace.",
            "This prompt targets DOWNSTREAM_APPLICATION_WORKSPACE.",
            "This prompt targets PARENT_DOMAIN_DISTRIBUTION_TEMPLATE.",
            "This prompt targets CHILD_DOMAIN_DISTRIBUTION_TEMPLATE.",
        ]
        for sample in valid_samples:
            self.assertTrue(
                check_repository_classification(sample),
                f"Expected classification to be accepted for: {sample!r}"
            )

    def test_repository_classification_rejection_when_absent_or_empty(self):
        """Verify check_repository_classification rejects absent or empty classification declarations."""
        invalid_samples = [
            "",
            "   ",
            "Task: Implement FEAT-01.\nTarget: scripts/dispatch_subagent.py",
            "Repository Classification:\nTarget: scripts/dispatch_subagent.py",
            "Repository Classification:   \nTarget: scripts/dispatch_subagent.py",
            "Repository Classification: \"\"\nTarget: scripts/dispatch_subagent.py",
            "Repository Classification: ''\nTarget: scripts/dispatch_subagent.py",
            "Classification:\nTarget: scripts/dispatch_subagent.py",
            "Repo Classification:   \n",
            "**Repository Classification:** \n",
        ]
        for sample in invalid_samples:
            self.assertFalse(
                check_repository_classification(sample),
                f"Expected classification to be rejected for: {sample!r}"
            )

    def test_validate_subagent_preflight_classification_acceptance_and_rejection(self):
        """Verify validate_subagent_preflight accepts valid workspace classifications and rejects empty."""
        base_prompt = """You are a context-isolated subagent.
{classification_line}
Target: scripts/dispatch_subagent.py

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits, commands, or tools.
2. Micro-Task Scope: Focus exclusively on target `scripts/dispatch_subagent.py`.
3. Defect Reporting: Record defects with `gh issue create` and `glab issue create`.

PROCEED
"""
        for cls_line in [
            "Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT",
            "Repository Classification: DOWNSTREAM_APPLICATION_WORKSPACE",
            "Repository Classification: PARENT_DOMAIN_DISTRIBUTION_TEMPLATE",
            "Repository Classification: CHILD_DOMAIN_DISTRIBUTION_TEMPLATE",
            "**Repository Classification**: `DOWNSTREAM_CUSTOMER_PROJECT`",
            "operating within classification `DOWNSTREAM_CUSTOMER_PROJECT`",
        ]:
            prompt = base_prompt.format(classification_line=cls_line)
            passed, reason = validate_subagent_preflight(prompt)
            self.assertTrue(passed, f"Expected pass for {cls_line!r}, got: {reason}")

        for empty_cls in [
            "Repository Classification:",
            "Repository Classification:   ",
            "Classification:",
            "**Repository Classification:**",
        ]:
            prompt = base_prompt.format(classification_line=empty_cls)
            passed, reason = validate_subagent_preflight(prompt)
            self.assertFalse(passed, f"Expected rejection for empty classification {empty_cls!r}")
            self.assertIn("missing repository classification", reason)


    def test_valid_prompt_with_step1_phrasing_variations(self):
        """Verify that prompts with valid step 1 variations pass linting."""
        variations = [
            "1. Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before executing any file edits, commands, or tools.",
            "1. Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as step 1 before performing any actions.",
            "1. Execute `view_file` as step 1 on `skills/spec-orchestrator/SKILL.md` before performing any actions.",
            "Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before taking any action.",
            "Prerequisite: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` before running actions.",
        ]
        base_step1 = "1. Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before performing any actions."
        for var in variations:
            prompt = self.valid_prompt.replace(base_step1, var)
            errors = lint_subagent_prompt(prompt)
            self.assertEqual(errors, [], f"Step 1 variation '{var}' had unexpected errors: {errors}")

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


class TestMandateFidelityGate(unittest.TestCase):
    """Corpus-sourced mandate fidelity gate tests (rules/subagent-dispatch-standards.md)."""

    @classmethod
    def setUpClass(cls):
        cls.requirements = TestSubagentPromptLinter._corpus_requirements()

    @staticmethod
    def _build(payload_rows):
        return TestSubagentPromptLinter._build_prompt(payload_rows)

    def test_mandate_fidelity_accepts_all_six_requirements_verbatim(self):
        """A payload containing all six Requirements verbatim must pass with zero lint errors."""
        payload = self._build(self.requirements)
        errors = lint_subagent_prompt(payload)
        self.assertEqual(errors, [])

    def test_mandate_fidelity_rejects_dropped_paraphrased_requirements_naming_them(self):
        """Dropping two Requirements and paraphrasing them must fail, naming exactly those two."""
        dropped = [
            "Contains verbatim repository classification",
            "Max 1 Epic, Feature, Story, or Use Case",
        ]
        paraphrased = [
            "States the repository class of the codebase",
            "Work on a single specification item at a time",
        ]
        rows = [r for r in self.requirements if r not in dropped] + paraphrased
        payload = self._build(rows)
        errors = lint_subagent_prompt(payload)
        fidelity_errors = [e for e in errors if "mandate fidelity violation" in e.lower()]
        self.assertEqual(len(fidelity_errors), 2)
        for expected in dropped:
            self.assertTrue(
                any(expected in e for e in fidelity_errors),
                f"missing Requirement '{expected}' not named in fidelity errors: {fidelity_errors}",
            )

    def test_mandate_fidelity_fails_closed_when_rules_corpus_unreadable(self):
        """The gate must fail closed when rules/subagent-dispatch-standards.md cannot be read."""
        backup_path = TestSubagentPromptLinter.RULES_PATH + ".unittest-backup"
        os.rename(TestSubagentPromptLinter.RULES_PATH, backup_path)
        try:
            errors = lint_subagent_prompt(self._build(self.requirements))
            self.assertTrue(
                any("subagent-dispatch-standards" in e for e in errors),
                f"expected fail-closed error naming the rules corpus, got: {errors}",
            )
        finally:
            os.rename(backup_path, TestSubagentPromptLinter.RULES_PATH)
        sane_errors = lint_subagent_prompt(self._build(self.requirements))
        self.assertEqual(sane_errors, [])

    def test_pre_dispatch_abort_path_halts_with_nonzero_exit(self):
        """A mocked outgoing payload failing the fidelity gate must HALT pre-write-out with non-zero exit."""
        from unittest import mock

        from scripts import dispatch_subagent

        laundered_payload = """
You are a context-isolated subagent operating under the DEAP Engineering Framework.

Role: Worker
Subagent Type: code_modifier_worker
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: src/module.py

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before executing any file edits, commands, or tools.
2. Micro-Task Scope: you may condense the instructions to save tokens.
3. Defect Reporting: use `gh issue create` (GitHub) or `glab issue create` (GitLab).

PROCEED
"""
        out_path = os.path.join(tempfile.gettempdir(), "dispatch_gate_test_out.md")
        if os.path.exists(out_path):
            os.remove(out_path)
        stderr_buf = io.StringIO()
        with mock.patch.object(dispatch_subagent, "generate_subagent_prompt", return_value=laundered_payload):
            with contextlib.redirect_stderr(stderr_buf):
                with self.assertRaises(SystemExit) as cm:
                    dispatch_subagent.dispatch_subagent(
                        skill="skills/spec-orchestrator/SKILL.md",
                        target="src/module.py",
                        output=out_path,
                    )
        self.assertNotEqual(cm.exception.code, 0)
        self.assertIn("HALT", stderr_buf.getvalue())
        self.assertFalse(
            os.path.exists(out_path),
            "payload file must not be written when the mandate fidelity gate HALTs pre-dispatch",
        )
        if os.path.exists(out_path):
            os.remove(out_path)


if __name__ == "__main__":
    unittest.main()
