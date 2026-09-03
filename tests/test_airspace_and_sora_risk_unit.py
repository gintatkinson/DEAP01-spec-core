#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for ConOps Unit 05 (Airspace & SORA Risk Assessment) per Issues #122 & #135.
Verifies:
1. Section 5.1.1 Ground Risk Buffer Parametric Wind Sensitivity Table (0-20 m/s sweep).
2. Section 5.2 Kinetic Impact Energy physics derivations (unmitigated free-fall, parachute equilibrium, failsafe mitigated energy, parameter table).
3. Section 5.4 SORA M1–M3 Mitigations Table (JARUS SORA v2.5, Low/Medium/High assurance, -1/-2 GRC credits).
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

UNIT_05_PATH = os.path.join(
    REPO_ROOT,
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "05_AIRSPACE_AND_SORA_RISK.md",
)


class TestAirspaceAndSoraRiskUnit(unittest.TestCase):
    """Test suite verifying 05_AIRSPACE_AND_SORA_RISK.md compliance (Issues #122 & #135)."""

    def setUp(self):
        self.assertTrue(os.path.isfile(UNIT_05_PATH), f"Target file missing: {UNIT_05_PATH}")
        with open(UNIT_05_PATH, "r", encoding="utf-8") as f:
            self.content = f.read()

    def test_section_5_1_1_wind_sensitivity_sweep(self):
        """Verify Section 5.1.1 contains parametric wind sensitivity table across 0 to 20 m/s in 5 m/s increments."""
        self.assertIn("### 5.1.1 Ground Risk Buffer Parametric Wind Sensitivity", self.content)

        # Mathematical formulation presence
        self.assertIn("t_{\\mathrm{fall}}", self.content)
        self.assertIn("d_{\\mathrm{wind}}", self.content)
        self.assertIn("d_{\\mathrm{impact}}", self.content)
        self.assertIn("\\Delta R", self.content)

        # Wind speed sweep points in table (0, 5, 10, 15, 20 m/s)
        for wind_speed in ["0.0", "5.0", "10.0", "15.0", "20.0"]:
            self.assertTrue(
                re.search(rf"\|\s*{re.escape(wind_speed)}\s*\|", self.content),
                f"Missing wind speed {wind_speed} m/s row in wind sensitivity table",
            )

        # Required metric columns in table
        for header_keyword in ["Wind Speed", "Fall Time", "Drift Distance", "Impact", "Margin"]:
            self.assertTrue(
                any(header_keyword.lower() in line.lower() for line in self.content.splitlines() if "|" in line),
                f"Missing column '{header_keyword}' in Section 5.1.1 table",
            )

    def test_section_5_2_kinetic_impact_energy_derivations(self):
        """Verify Section 5.2 contains physics derivations for unmitigated free-fall, parachute equilibrium, and mitigated energy."""
        self.assertIn("## 5. Operational State Space, Boundary Containment & Risk Assessment", self.content)
        self.assertIn("### 5.2", self.content)

        # Unmitigated free-fall kinetic impact derivation:
        # E_{k,\mathrm{unmitigated}} = \frac{1}{2} m v_{\mathrm{terminal,unmitigated}}^2 = \frac{m^2 g}{\rho S_{\mathrm{ref}} C_D}
        self.assertIn("E_{k,\\mathrm{unmitigated}}", self.content)
        self.assertIn("v_{\\mathrm{terminal,unmitigated}}", self.content)
        self.assertTrue(
            "\\frac{m^2 g}{\\rho S_{\\mathrm{ref}} C_D}" in self.content or
            "\\frac{m^2 g}{\\rho S_{\\mathrm{ref}} C_d}" in self.content or
            "\\frac{m^2 g}{\\rho S_{\\text{ref}} C_D}" in self.content,
            "Missing unmitigated kinetic energy physics derivation formula m^2*g / (rho*S_ref*C_D)",
        )

        # Parachute aerodynamic descent equilibrium derivation:
        # v_{\mathrm{terminal,parachute}} = \sqrt{\frac{2mg}{\rho S_{\mathrm{canopy}} C_{d,\mathrm{parachute}}}} \le 1.65 m/s
        self.assertIn("v_{\\mathrm{terminal,parachute}}", self.content)
        self.assertTrue(
            "\\sqrt{\\frac{2mg}{\\rho S_{\\mathrm{canopy}} C_{d,\\mathrm{parachute}}}}" in self.content or
            "\\sqrt{\\frac{2 m g}{\\rho S_{\\mathrm{canopy}} C_{d,\\mathrm{parachute}}}}" in self.content,
            "Missing parachute equilibrium terminal velocity derivation formula",
        )
        self.assertIn("1.65", self.content)

        # Failsafe-mitigated kinetic impact energy:
        # E_{k,\mathrm{mitigated}} = \frac{1}{2} m v_{\mathrm{terminal,parachute}}^2 \le 34.0 J
        self.assertIn("E_{k,\\mathrm{mitigated}}", self.content)
        self.assertIn("34.0", self.content)

        # Parameter definition table in 5.2
        param_symbols = [
            "m", "g", "rho", "S_ref", "C_D", "S_canopy", "C_d_parachute",
            "v_terminal_unmitigated", "E_k_unmitigated", "v_terminal_parachute", "E_k_mitigated"
        ]
        for sym in param_symbols:
            self.assertTrue(
                any(re.search(rf"\|\s*{re.escape(sym)}\s*\|", line) for line in self.content.splitlines()),
                f"Missing parameter '{sym}' in Section 5.2 parameter definition table",
            )

    def test_section_5_4_sora_m1_m3_mitigations_table(self):
        """Verify Section 5.4 SORA M1–M3 table with JARUS SORA v2.5 mitigations, Low/Medium/High assurance, and -1/-2 GRC credits."""
        self.assertIn("### 5.4", self.content)

        # M1, M2, M3 coverage
        self.assertTrue(re.search(r"M1", self.content), "Missing M1 mitigation")
        self.assertTrue(re.search(r"M2", self.content), "Missing M2 mitigation")
        self.assertTrue(re.search(r"M3", self.content), "Missing M3 mitigation")

        # Assurance Levels: Low, Medium, High
        self.assertIn("Low", self.content)
        self.assertIn("Medium", self.content)
        self.assertIn("High", self.content)

        # GRC Reduction step credits (-1 GRC, -2 GRC)
        self.assertIn("-1 GRC", self.content)
        self.assertIn("-2 GRC", self.content)

        # Citations
        self.assertIn("JARUS SORA v2.5", self.content)

    def test_katex_and_table_integrity(self):
        """Verify zero KaTeX rendering violations or table structure defects."""
        findings = check_katex_text(self.content, source="05_AIRSPACE_AND_SORA_RISK.md")
        self.assertEqual(
            findings,
            [],
            f"KaTeX / Markdown table integrity violations detected: {[str(f) for f in findings]}",
        )


if __name__ == "__main__":
    unittest.main()
