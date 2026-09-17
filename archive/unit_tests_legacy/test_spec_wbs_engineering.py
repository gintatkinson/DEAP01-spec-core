#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and integration tests for Work Breakdown Structure & Enterprise Realization Engineering (skills/spec-wbs-engineering/SKILL.md).
Verifies:
1. skills/spec-wbs-engineering/SKILL.md frontmatter, instruction steps, hierarchy, and schema mappings.
2. Mirror parity between skills/ and .agents/skills/.
3. 5-Tier WBS hierarchy with 7 concrete work packages per feature.
4. 7-column end-to-end traceability matrix schema.
5. Multi-platform Enterprise PM export rules (Jira, Monday.com, MS Project CSV and JSON AST).
6. Mathematical formatting, KaTeX rendering compliance, and Mermaid integrity.
"""

import os
import re
import sys
import unittest
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

WBS_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-wbs-engineering", "SKILL.md")
AGENTS_WBS_SKILL_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-wbs-engineering", "SKILL.md")


def _extract_yaml_frontmatter(content: str) -> dict:
    """Extract and parse YAML frontmatter from markdown text."""
    m = re.search(r"^---\s*\n(.*?)\n---\s*$", content, re.DOTALL | re.MULTILINE)
    if m:
        return yaml.safe_load(m.group(1)) or {}
    return {}


class TestSpecWbsEngineering(unittest.TestCase):
    """
    Test suite for spec-wbs-engineering skill specification.
    """

    def test_skill_files_exist_and_mirrored(self):
        """Verify that spec-wbs-engineering skill files exist on disk and match mirror."""
        self.assertTrue(os.path.isfile(WBS_SKILL_PATH), f"Missing {WBS_SKILL_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_WBS_SKILL_PATH), f"Missing {AGENTS_WBS_SKILL_PATH}")

        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f1, open(AGENTS_WBS_SKILL_PATH, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "Mirror mismatch between skills/ and .agents/skills/")

    def test_skill_frontmatter_structure(self):
        """Verify YAML frontmatter structure in spec-wbs-engineering/SKILL.md."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        fm = _extract_yaml_frontmatter(content)
        self.assertEqual(fm.get("name"), "spec-wbs-engineering")

        desc = fm.get("description", "")
        self.assertIn("MIL-STD-881E", desc)
        self.assertIn("Work Breakdown Structures", desc)
        self.assertIn("Technical Realization Registers", desc)
        self.assertIn("Jira", desc)
        self.assertIn("Monday.com", desc)
        self.assertIn("MS Project", desc)

        meta = fm.get("metadata", {})
        self.assertEqual(meta.get("title"), "Work Breakdown Structure & Enterprise Realization Engineering (Worker WBS)")
        self.assertEqual(meta.get("risk"), "medium")
        self.assertEqual(meta.get("source"), "custom")
        self.assertEqual(str(meta.get("version")), "1.0")

    def test_primary_commercial_toolchain_context(self):
        """Verify Primary Commercial Toolchain Integration Context is documented."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("MATLAB", content)
        self.assertIn("Simulink", content)
        self.assertIn("Stateflow", content)
        self.assertIn("Embedded Coder", content)
        self.assertIn("DO-178C", content)
        self.assertIn("SPARK Ada", content)

    def test_5_tier_wbs_decomposition_hierarchy(self):
        """Verify 5-tier WBS decomposition hierarchy and 7 concrete work packages per feature."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # 5 Tiers
        self.assertIn("Level 1: Program Root / Integrated System", content)
        self.assertIn("Level 2: Subsystem Segment Packages", content)
        self.assertIn("Level 3: Prime Mission Products / Domain Features", content)
        self.assertIn("Level 4/5: 7 Concrete Work Packages per Feature", content)

        # 7 Work Packages
        wps = [
            "WP-xxx-SPEC",
            "WP-xxx-MAT-PARAM",
            "WP-xxx-SL-BLD",
            "WP-xxx-PY-DOM",
            "WP-xxx-PY-ENG",
            "WP-xxx-TST",
            "WP-xxx-REP",
        ]
        for wp in wps:
            self.assertIn(wp, content, f"Missing work package {wp} in WBS skill documentation")

        # Target paths for WPs
        self.assertIn("docs/features/", content)
        self.assertIn("models/matlab/", content)
        self.assertIn("models/scripts/", content)
        self.assertIn("models/python/*_domain.py", content)
        self.assertIn("models/python/*_engine.py", content)
        self.assertIn("tests/test_feat_*", content)
        self.assertIn("docs/reports/simulink_results/", content)

    def test_7_column_traceability_matrix_schema(self):
        """Verify 7-column end-to-end traceability matrix schema."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        columns = [
            "SysML Component",
            "Feature Spec",
            "User Stories",
            "MATLAB / Simulink Plant",
            "Python 250 Hz Engine",
            "Verification Suite",
            "Simulation Evidence",
        ]
        for col in columns:
            self.assertIn(col, content, f"Missing traceability matrix column '{col}'")

    def test_enterprise_pm_export_rules(self):
        """Verify CSV schema for Jira/Monday/MS Project and JSON AST schema."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # CSV Export
        self.assertIn("wbs_export_jira_monday_ms_project.csv", content)
        self.assertIn("Jira Software", content)
        self.assertIn("Monday.com", content)
        self.assertIn("Microsoft Project", content)

        # JSON AST Export
        self.assertIn("wbs_export.json", content)
        self.assertIn("$schema", content)
        self.assertIn("WBS_Enterprise_Realization_AST", content)
        self.assertIn("traceability_matrix", content)

    def test_step_by_step_execution_workflow(self):
        """Verify all 5 execution steps are documented."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Step 1: Ingest System Root, ConOps, and Safety Baselines", content)
        self.assertIn("Step 2: Ingest Epics, Features, Use Cases, and User Stories", content)
        self.assertIn("Step 3: Discover Implementation Models & Verification Evidence", content)
        self.assertIn("Step 4: Synthesize Deliverables Suite", content)
        self.assertIn("Step 5: Verify Structural & Mathematical Consistency", content)

    def test_katex_rendering_integrity(self):
        """Verify LaTeX and KaTeX mathematical rendering syntax in WBS skill."""
        with open(WBS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
        cleaned = re.sub(r"`+.*?`+", "", cleaned)

        # Balanced $$ delimiters
        parts = cleaned.split("$$")
        self.assertEqual(
            (len(parts) - 1) % 2,
            0,
            f"Unbalanced $$ delimiters in {WBS_SKILL_PATH}",
        )

        # Balanced \begin{aligned} and \end{aligned}
        num_begin_aligned = len(re.findall(r"\\begin\{aligned\}", cleaned))
        num_end_aligned = len(re.findall(r"\\end\{aligned\}", cleaned))
        self.assertEqual(
            num_begin_aligned,
            num_end_aligned,
            f"Unbalanced \\begin{{aligned}} and \\end{{aligned}} in {WBS_SKILL_PATH}",
        )

        # No forbidden \begin{align}
        self.assertFalse(
            re.search(r"\\begin\{align\*?\}", cleaned),
            f"Forbidden \\begin{{align}} found in {WBS_SKILL_PATH}. Use \\begin{{aligned}} instead.",
        )

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")


if __name__ == "__main__":
    unittest.main()
