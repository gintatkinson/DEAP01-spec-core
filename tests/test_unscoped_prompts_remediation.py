#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for Issue #293 remediation:
Verifies that worker verification prompts across skills and catalogs mandate
the --only parameter to prevent whole-corpus audits during atomic tasks,
and that whole-workspace verify_model_coverage runs cleanly with exit code 0.

Realises: [Issue-293/UnscopedPromptsRemediation]
"""

import os
import re
import subprocess
import sys
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

from tests.test_prompt_catalog_integrity import extract_prompt_blocks


class TestUnscopedPromptsRemediation(unittest.TestCase):
    """Regression test suite for Issue #293 and whole-corpus audit prevention."""

    def test_spec_orchestrator_worker_prompts_mandate_only(self):
        """Verify Phase 1, Phase 2, and Phase 3 in spec-orchestrator/SKILL.md mandate --only <local-md-file>."""
        skill_path = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "SKILL.md")
        self.assertTrue(os.path.isfile(skill_path), f"Missing {skill_path}")

        with open(skill_path, "r", encoding="utf-8") as f:
            content = f.read()

        expected_command = './skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs --only "<local-md-file>"'

        # Extract Phase 1, Phase 2, Phase 3 sections
        phase1_match = re.search(r"## Phase 1: Structural Extraction.*?(?=## Phase 1\.5:)", content, re.DOTALL)
        self.assertIsNotNone(phase1_match, "Phase 1 section not found in spec-orchestrator/SKILL.md")
        self.assertIn(
            expected_command,
            phase1_match.group(0),
            "Phase 1 local validation command missing mandatory --only \"<local-md-file>\"",
        )

        phase2_match = re.search(r"## Phase 2 `\[P\]`: Behavioral Extraction.*?(?=## Phase 3:)", content, re.DOTALL)
        self.assertIsNotNone(phase2_match, "Phase 2 section not found in spec-orchestrator/SKILL.md")
        self.assertIn(
            expected_command,
            phase2_match.group(0),
            "Phase 2 local validation command missing mandatory --only \"<local-md-file>\"",
        )

        phase3_match = re.search(r"## Phase 3: System Interaction Extraction.*?(?=## Phase 4:)", content, re.DOTALL)
        self.assertIsNotNone(phase3_match, "Phase 3 section not found in spec-orchestrator/SKILL.md")
        self.assertIn(
            expected_command,
            phase3_match.group(0),
            "Phase 3 local validation command missing mandatory --only \"<local-md-file>\"",
        )

    def test_installer_worker_prompts_mandate_only(self):
        """Verify Worker 1A, 1C, 1D prompt templates in install_pipeline.sh mandate --only <spec_file>."""
        installer_path = os.path.join(REPO_ROOT, "scripts", "install_pipeline.sh")
        self.assertTrue(os.path.isfile(installer_path), f"Missing {installer_path}")

        with open(installer_path, "r", encoding="utf-8") as f:
            content = f.read()

        prompts = extract_prompt_blocks(content, r"### 4\.3")
        for worker_key in ["worker_1a", "worker_1b", "worker_1c"]:
            self.assertIn(worker_key, prompts, f"{worker_key} prompt not found in install_pipeline.sh")
            prompt = prompts[worker_key]
            self.assertRegex(
                prompt,
                r"verify_model_coverage\.py\s+--spec-only\s+--allow-missing-specs\s+--only\s+<spec_file>",
                f"{worker_key} prompt in install_pipeline.sh missing '--only <spec_file>' scoping parameter",
            )

    def test_readme_worker_prompts_mandate_only(self):
        """Verify Worker 1A, 1B, 1C prompt templates in README.md mandate --only <spec_file>."""
        readme_path = os.path.join(REPO_ROOT, "README.md")
        self.assertTrue(os.path.isfile(readme_path), f"Missing {readme_path}")

        with open(readme_path, "r", encoding="utf-8") as f:
            content = f.read()

        prompts = extract_prompt_blocks(content, r"### (4\.3|9\.2)") # Note: README uses 9.2 or 4.3 depending on version
        for worker_key in ["worker_1a", "worker_1b", "worker_1c"]:
            self.assertIn(worker_key, prompts, f"{worker_key} prompt not found in README.md")
            prompt = prompts[worker_key]
            self.assertRegex(
                prompt,
                r"verify_model_coverage\.py\s+--spec-only\s+--allow-missing-specs\s+--only\s+<spec_file>",
                f"{worker_key} prompt in README.md missing '--only <spec_file>' scoping parameter",
            )

    def test_skill_files_verify_model_coverage_mandate_only(self):
        """Verify that skill engineering documents specify --only <spec> for verify_model_coverage."""
        skills_to_check = [
            "spec-icd-engineering",
            "spec-user-story-engineering",
            "schema-specification-engineering",
            "spec-usecase-engineering",
        ]
        for skill_name in skills_to_check:
            skill_path = os.path.join(REPO_ROOT, "skills", skill_name, "SKILL.md")
            self.assertTrue(os.path.isfile(skill_path), f"Missing {skill_path}")
            with open(skill_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertRegex(
                content,
                r"verify_model_coverage\.py\s+--spec-only\s+--allow-missing-specs\s+--only\s+<spec>",
                f"verify_model_coverage in {skill_name}/SKILL.md does not mandate '--only <spec>'",
            )

    def test_verify_model_coverage_whole_workspace_exits_zero(self):
        """Verify that verify_model_coverage.py --spec-only --allow-missing-specs exits 0 across workspace."""
        cmd = [
            sys.executable,
            os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "scripts", "verify_model_coverage.py"),
            "--spec-only",
            "--allow-missing-specs",
        ]
        result = subprocess.run(
            cmd,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
        self.assertEqual(
            result.returncode,
            0,
            f"verify_model_coverage.py failed with exit code {result.returncode}:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}",
        )


if __name__ == "__main__":
    unittest.main()
