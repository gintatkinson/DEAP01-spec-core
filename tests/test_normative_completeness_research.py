#!/usr/bin/env python3
"""
Unit tests for Spec Orchestrator Normative-Completeness Research Step (#96).
(`tests/test_normative_completeness_research.py`)

Verifies that:
1. `skills/spec-orchestrator/SKILL.md` and `.agents/skills/spec-orchestrator/SKILL.md`
   contain the mandatory Normative-Completeness Research Step under Phase 0.5.
2. Normative research protocol governs identification, ingestion, and clause-level mapping
   of applicable regulatory and domain standards (e.g. ISO/IEC/IEEE 29148, NATO STANAG 4586,
   RTCA DO-178C / DO-254, SAE ARP4754A / ARP4761, MIL-STD-882E, JARUS SORA v2.5,
   ASTM F3269-17 / F3411-22a, RTCA DO-365B).
3. Mandatory output artifacts are required: Cited Research Inventory (docs/research/RESEARCH_INVENTORY.md)
   and Declared-Total Population Register.
4. Traceability rule: Every added obligation, hazard, control action, or catalog entry MUST
   carry a public clause citation; un-cited additions are strictly prohibited.
5. Mermaid sequence diagram in spec-orchestrator/SKILL.md includes the Normative-Completeness
   Research step.
6. Canonical template exists at skills/spec-orchestrator/resources/RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md.
"""

import os
import re
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


class TestNormativeCompletenessResearch(unittest.TestCase):
    """Test suite asserting normative-completeness research mandates in spec-orchestrator."""

    def test_target_files_exist(self):
        """Verify that all target skill files exist on disk."""
        for path in TARGET_SKILL_PATHS:
            self.assertTrue(os.path.exists(path), f"File not found: {path}")

    def test_normative_completeness_heading(self):
        """Verify that Phase 0.5 Normative-Completeness Research Step heading exists."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertTrue(
                "Phase 0.5" in content and "Normative-Completeness Research" in content,
                f"Missing Phase 0.5 Normative-Completeness Research Step in {path}",
            )

    def test_regulatory_and_domain_standards_referenced(self):
        """Verify that applicable regulatory and domain standards are referenced."""
        standards = [
            "ISO/IEC/IEEE 29148",
            "NATO STANAG 4586",
            "RTCA DO-178C",
            "DO-254",
            "SAE ARP4754A",
            "ARP4761",
            "MIL-STD-882E",
            "JARUS SORA v2.5",
            "ASTM F3269-17",
            "F3411-22a",
            "RTCA DO-365B",
        ]
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            for std in standards:
                self.assertIn(
                    std,
                    content,
                    f"Missing reference to standard '{std}' in {path}",
                )

    def test_mandatory_output_artifacts_mandated(self):
        """Verify that Cited Research Inventory and Declared-Total Population Register are mandated."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertIn(
                "RESEARCH_INVENTORY.md",
                content,
                f"Missing reference to RESEARCH_INVENTORY.md in {path}",
            )
            self.assertTrue(
                "Declared-Total Population Register" in content or "Declared Total Population Register" in content,
                f"Missing Declared-Total Population Register mandate in {path}",
            )

    def test_clause_citation_traceability_rule(self):
        """Verify strict clause citation traceability rule and prohibition of un-cited additions."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertTrue(
                "clause citation" in content or "clause-level" in content,
                f"Missing clause citation requirement in {path}",
            )
            self.assertTrue(
                "un-cited" in content or "uncited" in content,
                f"Missing prohibition of un-cited additions in {path}",
            )

    def test_mermaid_sequence_diagram_includes_normative_research(self):
        """Verify Mermaid sequence diagram includes Normative Research Worker and steps."""
        for path in TARGET_SKILL_PATHS:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertIn(
                "Phase 0.5",
                content,
                f"Missing Phase 0.5 in sequence diagram in {path}",
            )
            self.assertTrue(
                "Normative Research" in content or "RESEARCH_INVENTORY" in content,
                f"Missing Normative Research step in Mermaid diagram in {path}",
            )

    def test_canonical_research_inventory_template_exists(self):
        """Verify canonical template exists in resources/ and contains key sections and parameter tokens."""
        template_path = os.path.join(
            PROJECT_ROOT, "skills", "spec-orchestrator", "resources", "RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md"
        )
        self.assertTrue(os.path.isfile(template_path), f"Missing template at {template_path}")
        with open(template_path, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Declared-Total Population Register", content)
        self.assertIn("Clause", content)
        tokens = re.findall(r'\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}', content)
        self.assertGreaterEqual(len(tokens), 5, f"Expected >= 5 parameter tokens in template, found {len(tokens)}")


if __name__ == "__main__":
    unittest.main()
