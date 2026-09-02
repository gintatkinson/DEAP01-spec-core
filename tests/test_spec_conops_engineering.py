#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and integration tests for Hierarchical ConOps & Mission Intent Engineering (#113).
Verifies:
1. skills/spec-conops-engineering/SKILL.md frontmatter, instruction steps, and schema mappings.
2. rules/conops-mission-intent-integrity.md governance invariants.
3. skills/spec-orchestrator/SKILL.md Phase 0.75 orchestration and sequence diagram integration.
4. Mathematical formatting and KaTeX rendering compliance across new artifacts.
"""

import os
import re
import sys
import unittest
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

CONOPS_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "SKILL.md")
AGENTS_CONOPS_SKILL_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "SKILL.md")
CONOPS_RULE_PATH = os.path.join(REPO_ROOT, "rules", "conops-mission-intent-integrity.md")
ORCHESTRATOR_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "SKILL.md")


def _extract_yaml_frontmatter(content: str) -> dict:
    """Extract and parse YAML frontmatter from markdown text."""
    m = re.search(r"^---\s*\n(.*?)\n---\s*$", content, re.DOTALL | re.MULTILINE)
    if m:
        return yaml.safe_load(m.group(1)) or {}
    return {}


class TestSpecConopsEngineering(unittest.TestCase):
    """
    Test suite for spec-conops-engineering skill, rule, and orchestrator integration.
    """

    def test_skill_files_exist(self):
        """Verify that spec-conops-engineering skill files exist on disk."""
        self.assertTrue(os.path.isfile(CONOPS_SKILL_PATH), f"Missing {CONOPS_SKILL_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_SKILL_PATH), f"Missing {AGENTS_CONOPS_SKILL_PATH}")

    def test_skill_frontmatter_structure(self):
        """Verify YAML frontmatter structure in spec-conops-engineering/SKILL.md."""
        with open(CONOPS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        fm = _extract_yaml_frontmatter(content)
        self.assertEqual(fm.get("name"), "spec-conops-engineering")
        self.assertEqual(fm.get("version"), "1.0")

        desc = fm.get("description", "")
        self.assertIn("ISO/IEC/IEEE 29148:2018", desc)
        self.assertIn("INCOSE SE Handbook v5.0", desc)
        self.assertIn("NATO STANAG 4586", desc)
        self.assertIn("MIL-STD-882E", desc)
        self.assertIn("scripts/assemble_conops.py", desc)
        self.assertIn("docs/conops/units/conops/", desc)
        self.assertIn("docs/conops/units/mission_intent/", desc)

        meta = fm.get("metadata", {})
        self.assertEqual(meta.get("title"), "Hierarchical ConOps & Mission Intent Engineering")
        self.assertEqual(meta.get("category"), "specification")
        self.assertEqual(meta.get("risk"), "low")

    def test_skill_instruction_steps(self):
        """Verify complete instruction manual for Worker ConOps (Steps 1 through 6)."""
        with open(CONOPS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Step 1: Ingestion
        self.assertIn("Step 1", content)
        self.assertIn("RESEARCH_INVENTORY.md", content)
        self.assertIn("FMECA", content)
        self.assertIn("SORA", content)

        # Step 2: Discrete Unit Extraction
        self.assertIn("Step 2", content)
        self.assertIn("conops_specification_schema.json", content)
        self.assertIn("mission_intent_specification_schema.json", content)

        # 12 ConOps unit files
        conops_units = [
            "01_scope.md",
            "02_standards.md",
            "03_deficiencies.md",
            "04_capabilities.md",
            "05_lifecycle.md",
            "06_sora.md",
            "07_uaf_activities.md",
            "08_optx_matrix.md",
            "09_environments.md",
            "10_scenarios.md",
            "11_maintenance.md",
            "12_emergency_matrix.md",
        ]
        for unit in conops_units:
            self.assertIn(unit, content, f"Missing ConOps unit '{unit}' in skill documentation")

        # 10 Mission Intent unit files
        mission_units = [
            "01_intent.md",
            "02_metl.md",
            "03_moe_mop.md",
            "04_threats.md",
            "05_pace.md",
            "06_roe.md",
            "07_airspace.md",
            "08_gng.md",
            "09_bingo.md",
            "10_tags.md",
        ]
        for unit in mission_units:
            self.assertIn(unit, content, f"Missing Mission Intent unit '{unit}' in skill documentation")

        # Step 3: Standalone Unit Authoring
        self.assertIn("Step 3", content)
        self.assertIn("docs/conops/units/conops/", content)
        self.assertIn("docs/conops/units/mission_intent/", content)

        # Step 4: Open Schema and Architectural Invariants
        self.assertIn("Step 4", content)
        threat_domains = ["Kinetic", "Mechanical", "Environmental", "EW", "Cyber", "Power", "Thermal", "Optical", "Human"]
        for td in threat_domains:
            self.assertIn(td, content)

        # Step 5: Deterministic Assembly
        self.assertIn("Step 5", content)
        self.assertIn("assemble_conops.py", content)

        # Step 6: Multi-Gate Verification
        self.assertIn("Step 6", content)
        self.assertIn("Gate 26", content)
        self.assertIn("Gate 28", content)
        self.assertIn("Gate 29", content)

    def test_governance_rule_file_exists_and_valid(self):
        """Verify rules/conops-mission-intent-integrity.md exists and codifies mandatory invariants."""
        self.assertTrue(os.path.isfile(CONOPS_RULE_PATH), f"Missing {CONOPS_RULE_PATH}")
        with open(CONOPS_RULE_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Pure open schema contract
        self.assertTrue(
            "Pure Open Schema Contract" in content or "open schema" in content.lower(),
            "Missing open schema contract mandate in governance rule",
        )
        self.assertTrue(
            "Zero Static Row Caps" in content or "static row caps" in content.lower(),
            "Missing static row caps prohibition in governance rule",
        )

        # Multi-domain threat taxonomy (all 7 domains)
        domains = ["Kinetic", "Mechanical", "Environmental", "EW / Cyber", "Power / Thermal", "Optical", "Human"]
        for d in domains:
            self.assertIn(d, content, f"Missing threat domain '{d}' in governance rule")

        # INCOSE MoE/MoP formulation
        self.assertIn("INCOSE", content)
        self.assertIn("MoE", content)
        self.assertIn("MoP", content)
        self.assertIn("Threshold", content)
        self.assertIn("Objective", content)

        # Public clause citations
        self.assertTrue("100% Public Clause Citations" in content or "clause citation" in content.lower())

        # Gate 24 allocation tag
        self.assertIn("OperationalAllocation", content)

        # Standards referenced
        self.assertIn("ISO/IEC/IEEE 29148:2018", content)
        self.assertIn("NATO STANAG 4586", content)
        self.assertIn("MIL-STD-882E", content)
        self.assertIn("JARUS SORA v2.5", content)

    def test_orchestrator_phase_0_75_integration(self):
        """Verify spec-orchestrator/SKILL.md integrates Phase 0.75 in lifecycle and diagram."""
        with open(ORCHESTRATOR_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Heading presence
        self.assertIn("## Phase 0.75: Hierarchical ConOps & Mission Intent Tree Engineering", content)

        # Sequencing between Phase 0.5 and Phase 1
        pos_0_5 = content.find("## Phase 0.5:")
        pos_0_75 = content.find("## Phase 0.75:")
        pos_1 = content.find("## Phase 1:")

        self.assertTrue(pos_0_5 != -1, "Missing Phase 0.5 in spec-orchestrator")
        self.assertTrue(pos_0_75 != -1, "Missing Phase 0.75 in spec-orchestrator")
        self.assertTrue(pos_1 != -1, "Missing Phase 1 in spec-orchestrator")
        self.assertTrue(pos_0_5 < pos_0_75 < pos_1, "Phase 0.75 must be sequenced strictly between Phase 0.5 and Phase 1")

        # Participant in Mermaid sequence diagram
        self.assertIn('participant W_CO as "Phase 0.75: ConOps & Mission Intent Tree Worker"', content)
        self.assertIn("Phase 0.75 - ConOps & Mission Intent Tree Engineering", content)

        # Phase Worker subagent dispatch list
        self.assertIn("Phase 0.75: `ConOps & Mission Intent Tree Worker (Worker ConOps)`", content)

        # Phase 0.5 validation gate triggers Phase 0.75
        self.assertIn("execute Phase 0.75 immediately without pausing for user approval", content)

    def test_katex_rendering_integrity_across_new_files(self):
        """Verify LaTeX and KaTeX mathematical rendering syntax across newly created/modified markdown files."""
        files_to_check = [
            CONOPS_SKILL_PATH,
            CONOPS_RULE_PATH,
            ORCHESTRATOR_SKILL_PATH,
        ]

        for path in files_to_check:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()

            cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
            cleaned = re.sub(r"`+.*?`+", "", cleaned)

            # Balanced $$ math delimiters
            parts = cleaned.split("$$")
            self.assertEqual(
                (len(parts) - 1) % 2,
                0,
                f"Unbalanced $$ delimiters in {path} (found {len(parts) - 1} delimiters)",
            )

            # Balanced \begin{aligned} and \end{aligned}
            num_begin_aligned = len(re.findall(r"\\begin\{aligned\}", cleaned))
            num_end_aligned = len(re.findall(r"\\end\{aligned\}", cleaned))
            self.assertEqual(
                num_begin_aligned,
                num_end_aligned,
                f"Unbalanced \\begin{{aligned}} ({num_begin_aligned}) and \\end{{aligned}} ({num_end_aligned}) in {path}",
            )

            # No forbidden \begin{align}
            self.assertFalse(
                re.search(r"\\begin\{align\*?\}", cleaned),
                f"Forbidden \\begin{{align}} found in {path}. Use \\begin{{aligned}} instead.",
            )


if __name__ == "__main__":
    unittest.main()
