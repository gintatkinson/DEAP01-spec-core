#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for ConOps Unit 08 (Operational Environments & MIL-STD-810H Stress Qualification).
Fixes Issues #127, #134, #137.

Verifies:
1. Section 8.1 Master 12-Method Environmental Stress Qualification Table covering:
   M-500.6, M-501.7, M-502.7, M-503.7, M-505.7, M-506.6, M-507.6, M-509.7, M-510.7, M-514.8, M-516.8, M-521.4
   with Procedure Numbers, Operational Limits, Storage Limits, and Verification Standards.
2. Section 8.2 Granular Test Method Breakdowns (subsections 8.2.1 through 8.2.12) covering
   envelopes, durations, operational functional checks, and acceptance criteria for all 12 methods.
3. 100% domain agnosticism with {{PARAMETRIC_PLACEHOLDERS}} and zero hardcoded domain strings.
4. KaTeX mathematical rendering and table formatting integrity.
"""

import os
import re
import sys
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

PARITY_AUDITOR_SRC = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_AUDITOR_SRC not in sys.path:
    sys.path.insert(0, PARITY_AUDITOR_SRC)

from parity_auditor.validators.katex_validator import check_katex_text
from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS

UNIT_08_PATH = os.path.join(
    REPO_ROOT,
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
)

EXPECTED_12_METHODS = [
    ("M-500.6", "Low Pressure", "500.6"),
    ("M-501.7", "High Temperature", "501.7"),
    ("M-502.7", "Low Temperature", "502.7"),
    ("M-503.7", "Temperature Shock", "503.7"),
    ("M-505.7", "Solar Radiation", "505.7"),
    ("M-506.6", "Rain", "506.6"),
    ("M-507.6", "Humidity", "507.6"),
    ("M-509.7", "Salt Fog", "509.7"),
    ("M-510.7", "Sand and Dust", "510.7"),
    ("M-514.8", "Vibration", "514.8"),
    ("M-516.8", "Mechanical Shock", "516.8"),
    ("M-521.4", "Icing", "521.4"),
]


class TestEnvironmentalMilStd810HUnit(unittest.TestCase):
    """Test suite verifying 08_ENVIRONMENTAL_MIL_STD_810H.md compliance (Issues #127, #134, #137)."""

    def setUp(self):
        self.assertTrue(os.path.isfile(UNIT_08_PATH), f"Target file missing: {UNIT_08_PATH}")
        with open(UNIT_08_PATH, "r", encoding="utf-8") as f:
            self.content = f.read()

    def test_document_metadata_header_table(self):
        """Verify metadata attribute table and top-level heading."""
        self.assertTrue(self.content.startswith("| Attribute | Value |") or "<!--" in self.content[:100])
        self.assertIn("## 8. Operational Environments & MIL-STD-810H Environmental Stress Qualification", self.content)
        self.assertIn("Fixes: #127, #134, #137", self.content)

    def test_environmental_stress_vector_math_block(self):
        """Verify display KaTeX equation for ambient environmental state vector E_env."""
        self.assertIn(r"\mathbf{E}_{\mathrm{env}} &\in [\mathbf{E}_{\mathrm{min}}, \mathbf{E}_{\mathrm{max}}]", self.content)
        self.assertIn(r"\begin{aligned}", self.content)
        self.assertIn(r"\end{aligned}", self.content)
        self.assertIn("Parameter Definitions & Engineering Units:", self.content)
        for sym in ["E_env", "E_min", "E_max", "P_amb", "T_amb", "T_gradient", "I_solar", "R_precip", "RH_ambient", "C_salt", "C_particulate", "S_vib(f)", "a_shock", "delta_ice", "E_EMC"]:
            self.assertIn(sym, self.content)

    def test_section_8_1_master_12_method_table(self):
        """Verify Section 8.1 Master 12-Method Environmental Stress Qualification Table."""
        self.assertIn("### 8.1 Master 12-Method Environmental Stress Qualification Table", self.content)

        # Table headers
        for hdr in ["Method ID", "Environmental Stress Method Name", "Procedure Numbers", "Operational Limits", "Storage / Transit Limits", "Verification Standards"]:
            self.assertIn(hdr, self.content)

        # All 12 methods in Section 8.1 table
        for method_id, method_name, clause in EXPECTED_12_METHODS:
            pattern = rf"\|\s*\*\*{re.escape(method_id)}\*\*\s*\|\s*{re.escape(method_name)}"
            self.assertTrue(
                re.search(pattern, self.content, re.IGNORECASE),
                f"Missing method {method_id} ({method_name}) in Section 8.1 table",
            )

    def test_section_8_2_granular_test_method_breakdowns_all_12_subsections(self):
        """Verify Section 8.2 Granular Test Method Breakdowns (subsections 8.2.1 through 8.2.12)."""
        self.assertIn("### 8.2 Granular Test Method Breakdowns", self.content)

        for idx in range(1, 13):
            sub_num = f"8.2.{idx}"
            self.assertTrue(
                any(line.strip().startswith(f"#### {sub_num}") for line in self.content.splitlines()),
                f"Missing subsection #### {sub_num} in Section 8.2",
            )

        # Verify all 12 methods have detailed breakdown elements
        for idx, (method_id, method_name, clause) in enumerate(EXPECTED_12_METHODS, start=1):
            sub_heading = f"8.2.{idx}"
            sub_sec_match = re.search(rf"####\s+{re.escape(sub_heading)}[^\n]*\n([\s\S]*?)(?=####|\n---\n|###|$)", self.content)
            self.assertIsNotNone(sub_sec_match, f"Could not extract subsection {sub_heading}")
            sub_text = sub_sec_match.group(1)

            self.assertIn("- **Applicable Procedures:**", sub_text, f"Missing Applicable Procedures in {sub_heading}")
            self.assertIn("- **Environmental Envelope:**", sub_text, f"Missing Environmental Envelope in {sub_heading}")
            self.assertIn("- **Exposure Duration:**", sub_text, f"Missing Exposure Duration in {sub_heading}")
            self.assertIn("- **Operational Functional Checks:**", sub_text, f"Missing Operational Functional Checks in {sub_heading}")
            self.assertIn("- **Acceptance Criteria:**", sub_text, f"Missing Acceptance Criteria in {sub_heading}")

    def test_subsections_8_3_8_4_8_5_presence(self):
        """Verify Ingress Protection (8.3), EMC/EMI (8.4), and Physical Spatial Constraints (8.5)."""
        self.assertIn("### 8.3 Ingress Protection (IEC 60529) & Environmental Sealing Architecture", self.content)
        self.assertIn("### 8.4 Electromagnetic Compatibility (EMC/EMI) & RF Environments", self.content)
        self.assertIn("### 8.5 Physical Spatial Constraints & Deployment Envelopes", self.content)

    def test_domain_agnosticism_and_parametric_placeholders(self):
        """Verify 100% domain agnosticism with {{PARAMETRIC_PLACEHOLDERS}} and zero forbidden domain nouns."""
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, self.content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in 08_ENVIRONMENTAL_MIL_STD_810H.md: {violations}")

        placeholders = re.findall(r"\{\{[A-Za-z0-9_]+\}\}", self.content)
        self.assertGreaterEqual(len(placeholders), 30, f"Expected >= 30 parametric placeholders, found {len(placeholders)}")

    def test_katex_and_markdown_table_integrity(self):
        """Verify zero KaTeX rendering violations and no LaTeX math delimiters inside table cells."""
        # Math delimiters in table lines check
        table_lines = [line for line in self.content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        findings = check_katex_text(self.content, source="08_ENVIRONMENTAL_MIL_STD_810H.md")
        self.assertEqual(
            findings,
            [],
            f"KaTeX / Markdown table integrity violations detected: {[str(f) for f in findings]}",
        )


if __name__ == "__main__":
    unittest.main()
