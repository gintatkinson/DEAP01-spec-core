#!/usr/bin/env python3
"""
Unit tests for Spec Orchestrator Error Recovery & Mandatory Adversarial Audit Protocol.
(`tests/test_spec_orchestrator_error_recovery.py`)

Verifies that both `skills/spec-orchestrator/SKILL.md` and
`.agents/skills/spec-orchestrator/SKILL.md` contain the mandatory adversarial audit
recovery protocol under `## Error Recovery & Mandatory Adversarial Audit`.
"""

import os
import sys
import unittest

TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

TARGET_SKILL_PATHS = [
    os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "SKILL.md"),
    os.path.join(PROJECT_ROOT, ".agents", "skills", "spec-orchestrator", "SKILL.md"),
]


class TestSpecOrchestratorErrorRecovery(unittest.TestCase):
    """Test suite asserting error recovery and adversarial audit mandates in spec-orchestrator skills."""

    def test_target_files_exist(self):
        """Verify that all target skill files exist on disk."""
        for path in TARGET_SKILL_PATHS:
            self.assertTrue(os.path.exists(path), f"File not found: {path}")

    def test_error_recovery_section_heading(self):
        """Verify that '## Error Recovery & Mandatory Adversarial Audit' section heading exists."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertIn(
                "## Error Recovery & Mandatory Adversarial Audit",
                content,
                f"Missing section '## Error Recovery & Mandatory Adversarial Audit' in {path}",
            )

    def test_no_ad_hoc_triage_mandate(self):
        """Verify the mandate prohibiting ad-hoc triage and raw CLI dumps upon failure."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertTrue(
                "ad-hoc triage" in content or "ad hoc triage" in content,
                f"Missing prohibition of ad-hoc triage in {path}",
            )
            self.assertTrue(
                "raw CLI dump" in content or "raw CLI dumps" in content or "CLI dumps" in content,
                f"Missing prohibition of raw CLI dumps in {path}",
            )

    def test_adversarial_auditor_subagent_dispatch(self):
        """Verify that dispatch of Adversarial Code Auditor subagent (adversarial_auditor) is mandated."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertIn(
                "Adversarial Code Auditor",
                content,
                f"Missing Role 'Adversarial Code Auditor' in {path}",
            )
            self.assertIn(
                "adversarial_auditor",
                content,
                f"Missing TypeName 'adversarial_auditor' in {path}",
            )
            self.assertIn(
                "skills/adversarial-code-auditor/SKILL.md",
                content,
                f"Missing reference to 'skills/adversarial-code-auditor/SKILL.md' in {path}",
            )

    def test_adversarial_audit_pillars_and_requirements(self):
        """Verify 5 Whys, 4-pillar audit, offline Mermaid verification, and 7-section defect report."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertIn(
                "5 Whys",
                content,
                f"Missing 5 Whys root cause analysis mandate in {path}",
            )
            self.assertTrue(
                "4-pillar" in content or "4 pillars" in content or "four pillars" in content or "four-pillar" in content,
                f"Missing 4-pillar audit mandate in {path}",
            )
            self.assertTrue(
                "Mermaid" in content and ("offline" in content or "offline Mermaid" in content),
                f"Missing offline Mermaid verification in {path}",
            )
            self.assertTrue(
                "7-section" in content or "7 sections" in content or "seven-section" in content,
                f"Missing 7-section defect report mandate in {path}",
            )

    def test_issue_tracker_filing_mandate(self):
        """Verify issue creation commands for defect submission via file_defect.py."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertTrue(
                "file_defect.py" in content and "--label" in content and "bug" in content,
                f"Missing 'file_defect.py' defect filing mandate with '--label bug' in {path}",
            )

    def test_debug_protocol_and_tdd_remediation(self):
        """Verify dispatch to debug protocol (code_modifier_worker) for TDD RED-GREEN fix."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertTrue(
                "debug-protocol" in content or "skills/debug-protocol/SKILL.md" in content,
                f"Missing debug protocol reference in {path}",
            )
            self.assertIn(
                "code_modifier_worker",
                content,
                f"Missing 'code_modifier_worker' dispatch reference in {path}",
            )
            self.assertTrue(
                "RED-GREEN" in content or "RED-GREEN-REFACTOR" in content,
                f"Missing TDD RED-GREEN fix reference in {path}",
            )


if __name__ == "__main__":
    unittest.main()
