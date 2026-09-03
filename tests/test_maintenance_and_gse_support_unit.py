#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for ConOps Unit 10 (Maintenance & Support Equipment Concepts).
Fixes Issues #124, #140, #141.

Verifies:
1. Section 10.1.1 Formal Maintenance Task Cards Table (MTC-01, MTC-01A, MTC-02, MTC-03, MTC-04, MTC-05)
   covering scope, tier, trigger/interval, target SLA duration, qualification/tooling, sign-off authority/verification protocol, and public clause citations.
2. Section 10.1.2 Rapid Sortie Turnaround Workflow Diagram (Mermaid flowchart TD) and 7-step execution protocol for 15-minute SLA.
3. Section 10.1.3 Tool-less Modular LRU Replacement Steps for Flight Computer (t <= 5 min), Battery/Energy Module (t <= 2 min), Primary Sensor Payload (t <= 8 min), and Actuator/Motor Assembly (t <= 10 min).
4. Section 10.2 Support Equipment (SE) Taxonomy Table (SE-01 through SE-06).
5. Section 10.2.1 ISO/IEC 17025 GSE Calibration Matrix Table covering calibration intervals, standards baselines, allowable tolerances, and recalibration triggers for SE-01..SE-06 and FTK-01.
6. 100% domain agnosticism with {{PARAMETRIC_PLACEHOLDERS}} and zero hardcoded domain strings.
7. KaTeX mathematical rendering and table formatting integrity (no LaTeX math delimiters inside table cells).
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

UNIT_10_PATH = os.path.join(
    REPO_ROOT,
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
)


class TestMaintenanceAndGseSupportUnit(unittest.TestCase):
    """Test suite verifying 10_MAINTENANCE_AND_GSE_SUPPORT.md compliance (Issues #124, #140, #141)."""

    def setUp(self):
        self.assertTrue(os.path.isfile(UNIT_10_PATH), f"Target file missing: {UNIT_10_PATH}")
        with open(UNIT_10_PATH, "r", encoding="utf-8") as f:
            self.content = f.read()

    def test_document_metadata_header_table(self):
        """Verify metadata attribute table, top-level heading, and issue traceability baseline."""
        self.assertTrue(self.content.startswith("| Attribute | Value |"))
        self.assertIn("## 10. Maintenance & Support Equipment (SE) Concepts", self.content)
        self.assertIn("Fixes #124, #140, #141", self.content)

    def test_three_tier_maintenance_model_and_intro(self):
        """Verify Section 10.1 Three-Tier Maintenance Model (O-Level, I-Level, D-Level)."""
        self.assertIn("### 10.1 Three-Tier Maintenance Model (O-Level, I-Level, D-Level)", self.content)
        self.assertIn("ISO/IEC/IEEE 29148:2018 §5.2.4", self.content)
        self.assertIn("INCOSE SEH v5.0 §3.2", self.content)
        self.assertIn("MIL-HDBK-470A", self.content)
        self.assertIn("MIL-STD-882E", self.content)
        for tier in ["O-Level (Organizational)", "I-Level (Intermediate)", "D-Level (Depot / Factory)"]:
            self.assertIn(tier, self.content)

    def test_section_10_1_1_formal_maintenance_task_cards_table(self):
        """Verify Section 10.1.1 Maintenance Task Cards Table for MTC-01 through MTC-05."""
        self.assertIn("#### 10.1.1 Formal Maintenance Task Cards (MTC-01 through MTC-05)", self.content)

        # Table header columns
        table_headers = [
            "Task Card ID",
            "Task Card Title & Scope",
            "Maintenance Tier",
            "Execution Trigger & Interval",
            "Target SLA Duration",
            "Required Qualifications & Tooling",
            "Sign-Off Authority & Verification Protocol",
            "Public Clause Citation",
        ]
        for hdr in table_headers:
            self.assertIn(hdr, self.content)

        # All 6 task cards
        task_cards = [
            ("MTC-01", "Pre-Sortie / Pre-Operation Inspection", "O-Level"),
            ("MTC-01A", "Rapid Sortie Turnaround (15-min SLA)", "O-Level"),
            ("MTC-02", "Scheduled 50-Hour Phase Check", "I-Level"),
            ("MTC-03", "100-Hour Major Overhaul", "D-Level"),
            ("MTC-04", "Unscheduled Field LRU Swap", "I-Level / O-Level"),
            ("MTC-05", "Post-Incident Blackbox Quarantine", "I-Level / Safety Authority"),
        ]
        for card_id, title_snippet, tier in task_cards:
            pattern = rf"\|\s*\*\*{re.escape(card_id)}\*\*\s*\|\s*{re.escape(title_snippet)}"
            self.assertTrue(
                re.search(pattern, self.content, re.IGNORECASE),
                f"Missing task card {card_id} ({title_snippet}) in Section 10.1.1 table",
            )

    def test_section_10_1_2_rapid_sortie_turnaround_workflow_and_protocol(self):
        """Verify Section 10.1.2 Mermaid flowchart and 7-step execution protocol for 15-minute SLA."""
        self.assertIn("#### 10.1.2 Rapid Sortie Turnaround Workflow & 7-Step Protocol", self.content)
        self.assertIn("flowchart TD", self.content)

        # Mermaid flowchart nodes
        self.assertIn("Step 1: System Ingress & Safe-State Disarm", self.content)
        self.assertIn("Step 2: Telemetry Data Offload & Log Audit", self.content)
        self.assertIn("Step 3: Rapid Energy Module Hot-Swap", self.content)
        self.assertIn("Step 4: Rapid Visual & Structural Inspection", self.content)
        self.assertIn("Step 5: Mission Re-Tasking & Key Injection", self.content)
        self.assertIn("Step 6: Automated PBIT Diagnostics", self.content)
        self.assertIn("Step 7: Final Arming & Sortie Release", self.content)
        self.assertIn("Abort Turnaround & Route to MTC-04 LRU Swap", self.content)

        # 7-Step detailed protocol
        for step_num in range(1, 8):
            self.assertIn(f"**Step {step_num}:", self.content)

    def test_section_10_1_3_tool_less_lru_replacement_steps(self):
        """Verify Section 10.1.3 tool-less modular LRU table and step-by-step procedures."""
        self.assertIn("#### 10.1.3 Tool-less Modular Line Replaceable Unit (LRU) Replacement Procedures", self.content)

        # LRU table checks
        lrus = [
            ("LRU-01: Core Guidance Computer", "Ruggedized ZIF Cam-Lock Backplane", "Tool-less (Manual Cam Lever)"),
            ("LRU-02: Battery / Energy Module", "Polarized Keyed Blind-Mate Rail", "Tool-less (Spring Detent Latch)"),
            ("LRU-03: Primary Sensor Payload", "Kinematic Mount with Bayonet Collar", "Tool-less (Quick-Disconnect Ring)"),
            ("LRU-04: Actuator / Motor Assembly", "Precision Index Dowel & Bayonet Ring", "Tool-less (Positive-Stop Clamp)"),
        ]
        for lru_id, iface, tooling in lrus:
            self.assertIn(lru_id, self.content)
            self.assertIn(iface, self.content)
            self.assertIn(tooling, self.content)

        # Detailed step-by-step procedures
        self.assertIn("Core Guidance Computer Module Replacement", self.content)
        self.assertIn("Battery / Energy Module Replacement", self.content)
        self.assertIn("Primary Sensor Payload Replacement", self.content)
        self.assertIn("Actuator / Motor Assembly Replacement", self.content)

    def test_section_10_2_support_equipment_taxonomy_table(self):
        """Verify Section 10.2 Support Equipment Taxonomy Table for SE-01 through SE-06."""
        self.assertIn("### 10.2 Support Equipment (SE) Taxonomy", self.content)

        # SE table headers
        for hdr in ["SE Identifier", "SE Nomenclature", "Functional Purpose & Capabilities", "Operating Constraints & Ratings", "Public Standard Baseline"]:
            self.assertIn(hdr, self.content)

        # SE items SE-01 through SE-06
        se_items = [
            ("SE-01", "Multi-Bay Intelligent Resource Management Hub", "IEC 62133-2 / UN 38.3"),
            ("SE-02", "Ruggedized Field Control Terminal", "MIL-STD-810H / NIST SP 800-82r3"),
            ("SE-03", "Telescoping Antenna Transceiver Mast", "MIL-STD-810H / IEEE Std 1558-2020"),
            ("SE-04", "Environmental Diagnostic Test Unit", "ISO/IEC 17025 / MIL-STD-810H"),
            ("SE-05", "Precision Sensor Alignment Rig", "ISO/IEC 17025 §7.6 / IEEE Std 1558-2020"),
            ("SE-06", "Automated Calibration & Metrology Bench", "ISO/IEC 17025:2017 / NIST Traceable"),
        ]
        for se_id, nom, std in se_items:
            pattern = rf"\|\s*\*\*{re.escape(se_id)}\*\*\s*\|\s*{re.escape(nom)}"
            self.assertTrue(
                re.search(pattern, self.content, re.IGNORECASE),
                f"Missing {se_id} ({nom}) in Section 10.2 table",
            )
            self.assertIn(std, self.content)

    def test_section_10_2_1_calibration_matrix_table_and_invariants(self):
        """Verify Section 10.2.1 ISO/IEC 17025 Calibration Matrix Table for SE-01..SE-06, FTK-01."""
        self.assertIn("#### 10.2.1 ISO/IEC 17025 Support Equipment Calibration Matrix", self.content)

        # Headers
        matrix_headers = [
            "Equipment ID",
            "Equipment Nomenclature",
            "Parameter / Physical Quantity Measured",
            "Calibration Interval",
            "Reference Standards Baseline",
            "Allowable Tolerance Limits",
            "Recalibration Triggers",
        ]
        for hdr in matrix_headers:
            self.assertIn(hdr, self.content)

        # Items
        items = ["SE-01", "SE-02", "SE-03", "SE-04", "SE-05", "SE-06", "FTK-01"]
        for item_id in items:
            self.assertIn(f"**{item_id}**", self.content)

        # Invariants
        self.assertIn("Metrological Traceability Invariant", self.content)
        self.assertIn("Tamper-Evident Certification Labeling", self.content)
        self.assertIn("Out-of-Tolerance Quarantine Protocol", self.content)

    def test_subsections_10_3_10_4_10_5_presence(self):
        """Verify calibration fixtures (10.3), transit cases (10.4), and tooling/spares (10.5)."""
        self.assertIn("### 10.3 Calibration Fixtures & Sensor Alignment Rigs", self.content)
        self.assertIn("### 10.4 Ruggedized Transit Cases & Environmental Storage", self.content)
        self.assertIn("### 10.5 Field Maintenance Tooling & Spares Provisioning", self.content)
        self.assertIn("Standard Field Tool Kit (FTK-01)", self.content)
        self.assertIn("Authorized Field Spares Kit (FSK-01)", self.content)

    def test_domain_agnosticism_and_parametric_placeholders(self):
        """Verify 100% domain agnosticism with {{PARAMETRIC_PLACEHOLDERS}} and zero forbidden domain nouns."""
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, self.content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in 10_MAINTENANCE_AND_GSE_SUPPORT.md: {violations}")

        placeholders = re.findall(r"\{\{[A-Za-z0-9_]+\}\}", self.content)
        self.assertGreaterEqual(len(placeholders), 10, f"Expected >= 10 parametric placeholders, found {len(placeholders)}")

    def test_katex_and_markdown_table_integrity(self):
        """Verify zero KaTeX rendering violations and no LaTeX math delimiters inside table cells."""
        table_lines = [line for line in self.content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        findings = check_katex_text(self.content, source="10_MAINTENANCE_AND_GSE_SUPPORT.md")
        self.assertEqual(
            findings,
            [],
            f"KaTeX / Markdown table integrity violations detected: {[str(f) for f in findings]}",
        )


if __name__ == "__main__":
    unittest.main()
