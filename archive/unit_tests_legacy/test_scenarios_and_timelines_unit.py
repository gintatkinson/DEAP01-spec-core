#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for ConOps Unit 09 (Multi-Threaded Operational Scenarios & System Timelines).
Fixes Issues #126, #131, #138.

Verifies:
1. Scenario SCN-01 Nominal Lifecycle Thread with full 8-step timeline table with
   elapsed time (T+), Stimulus/Trigger, Actor, Action, Telemetry Stream, Decision Gate,
   Exception Branch, and Exit Criterion.
2. Scenario SCN-02 High-Throughput State Tracking & Target Processing with 6-step timeline table.
3. Scenario SCN-03 Degraded C2 Lost-Link & Autonomous Fallback Return with 6-step timeline table.
4. Scenario SCN-04 Dynamic Geofence Boundary Divert with 6-step timeline table.
5. Scenario SCN-05 Controlled Safety Interlock Action with Mermaid stateDiagram-v2 state machine
   and 4 decomposed phase execution verification tables (Ingress & Station Keeping,
   Positive Identification & Interlock Check, Dual-Consent Arming & Execution, Post-Action Assessment & Telemetry Dump).
6. 100% domain agnosticism with zero hardcoded domain strings and zero forbidden domain nouns.
7. KaTeX mathematical rendering and table formatting integrity.
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

UNIT_09_PATH = os.path.join(
    REPO_ROOT,
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "09_SCENARIOS_AND_TIMELINES.md",
)
AGENTS_UNIT_09_PATH = os.path.join(
    REPO_ROOT,
    ".agents",
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "09_SCENARIOS_AND_TIMELINES.md",
)


class TestScenariosAndTimelinesUnit(unittest.TestCase):
    """Test suite verifying 09_SCENARIOS_AND_TIMELINES.md compliance (Issues #126, #131, #138)."""

    def setUp(self):
        self.assertTrue(os.path.isfile(UNIT_09_PATH), f"Target file missing: {UNIT_09_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_UNIT_09_PATH), f"Mirror file missing: {AGENTS_UNIT_09_PATH}")
        with open(UNIT_09_PATH, "r", encoding="utf-8") as f:
            self.content = f.read()
        with open(AGENTS_UNIT_09_PATH, "r", encoding="utf-8") as f:
            self.mirror_content = f.read()

    def test_file_mirroring_integrity(self):
        """Verify skills/ and .agents/ copies of 09_SCENARIOS_AND_TIMELINES.md are identical."""
        self.assertEqual(self.content, self.mirror_content, "Mismatch between skills/ and .agents/ copies of Unit 09.")

    def test_document_metadata_header_table(self):
        """Verify metadata attribute table and top-level heading."""
        self.assertTrue(self.content.startswith("| Attribute | Value |"))
        self.assertIn("## 9. Multi-Threaded Operational Scenarios & System Timelines", self.content)

    def test_scenario_scn_01_nominal_lifecycle_8_step_table(self):
        """Verify Scenario SCN-01 Nominal Lifecycle Thread with full 8-step timeline table (Issue #126)."""
        self.assertIn("### 9.1 Scenario SCN-01: Nominal Lifecycle Thread", self.content)

        # Extract SCN-01 section
        sec1_match = re.search(r"### 9\.1 Scenario SCN-01[\s\S]*?(?=### 9\.2)", self.content)
        self.assertIsNotNone(sec1_match, "SCN-01 section not found")
        sec1_text = sec1_match.group(0)

        # Mandatory columns
        for col in [
            "Step Number",
            "Elapsed Time (T+)",
            "Stimulus / Trigger",
            "Actor / Performer",
            "Action Executed",
            "Telemetry Stream",
            "Decision Gate",
            "Exception Branch",
            "Exit Criterion",
        ]:
            self.assertIn(col, sec1_text)

        # All 8 sequential steps
        for step_idx in range(1, 9):
            self.assertIn(f"**{step_idx}**", sec1_text, f"Missing step {step_idx} in SCN-01")

        # Key decision gates in SCN-01
        self.assertIn("Gate GNG-01", sec1_text)
        self.assertIn("Gate GNG-02", sec1_text)
        self.assertIn("Gate GNG-03", sec1_text)
        self.assertIn("Gate GNG-04", sec1_text)
        self.assertIn("Gate GNG-05", sec1_text)
        self.assertIn("Gate GNG-06", sec1_text)
        self.assertIn("Gate GNG-07", sec1_text)
        self.assertIn("Gate GNG-08", sec1_text)

    def test_scenario_scn_02_high_throughput_tracking_6_step_table(self):
        """Verify Scenario SCN-02 High-Throughput State Tracking & Target Processing with 6-step table (Issue #131)."""
        self.assertIn("### 9.2 Scenario SCN-02: High-Throughput State Tracking & Target Processing", self.content)

        sec2_match = re.search(r"### 9\.2 Scenario SCN-02[\s\S]*?(?=### 9\.3)", self.content)
        self.assertIsNotNone(sec2_match, "SCN-02 section not found")
        sec2_text = sec2_match.group(0)

        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec2_text, f"Missing step {step_idx} in SCN-02")

        # Gates GNG-T1 through GNG-T6
        for gate_idx in range(1, 7):
            self.assertIn(f"Gate GNG-T{gate_idx}", sec2_text)

    def test_scenario_scn_03_degraded_c2_lost_link_6_step_table(self):
        """Verify Scenario SCN-03 Degraded C2 Lost-Link & Autonomous Fallback Return with 6-step table (Issue #131)."""
        self.assertIn("### 9.3 Scenario SCN-03: Degraded C2 Lost-Link & Autonomous Fallback Return", self.content)

        sec3_match = re.search(r"### 9\.3 Scenario SCN-03[\s\S]*?(?=### 9\.4)", self.content)
        self.assertIsNotNone(sec3_match, "SCN-03 section not found")
        sec3_text = sec3_match.group(0)

        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec3_text, f"Missing step {step_idx} in SCN-03")

        # Gates GNG-C1 through GNG-C6
        for gate_idx in range(1, 7):
            self.assertIn(f"Gate GNG-C{gate_idx}", sec3_text)

    def test_scenario_scn_04_dynamic_geofence_divert_6_step_table(self):
        """Verify Scenario SCN-04 Dynamic Geofence Boundary Divert with 6-step table (Issue #131)."""
        self.assertIn("### 9.4 Scenario SCN-04: Dynamic Geofence Boundary Divert", self.content)

        sec4_match = re.search(r"### 9\.4 Scenario SCN-04[\s\S]*?(?=### 9\.5)", self.content)
        self.assertIsNotNone(sec4_match, "SCN-04 section not found")
        sec4_text = sec4_match.group(0)

        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec4_text, f"Missing step {step_idx} in SCN-04")

        # Gates GNG-D1 through GNG-D6
        for gate_idx in range(1, 7):
            self.assertIn(f"Gate GNG-D{gate_idx}", sec4_text)

    def test_scenario_scn_05_safety_interlock_statemachine_and_4_phase_tables(self):
        """Verify Scenario SCN-05 with Mermaid state machine and 4 decomposed phase verification tables (Issue #138)."""
        self.assertIn("### 9.5 Scenario SCN-05: Controlled Safety Interlock Action", self.content)

        sec5_match = re.search(r"### 9\.5 Scenario SCN-05[\s\S]*$", self.content)
        self.assertIsNotNone(sec5_match, "SCN-05 section not found")
        sec5_text = sec5_match.group(0)

        # Mermaid stateDiagram-v2
        self.assertIn("stateDiagram-v2", sec5_text)
        self.assertIn("Phase_Ingress_StationKeeping", sec5_text)
        self.assertIn("Phase_PID_InterlockCheck", sec5_text)
        self.assertIn("Phase_DualConsent_ArmingExecution", sec5_text)
        self.assertIn("Phase_PostAction_AssessmentDump", sec5_text)

        # 4 decomposed phase execution verification tables
        phase_subheadings = [
            "#### 9.5.1 Safety Interlock State Machine & Deterministic Transitions",
            "#### 9.5.2 Phase 1: Ingress & Station Keeping Execution Verification",
            "#### 9.5.3 Phase 2: Positive Identification & Interlock Check Execution Verification",
            "#### 9.5.4 Phase 3: Dual-Consent Arming & Execution Verification",
            "#### 9.5.5 Phase 4: Post-Action Assessment & Telemetry Dump Execution Verification",
        ]
        for subh in phase_subheadings:
            self.assertIn(subh, sec5_text)

        # Verify each phase table has steps 1 to 4
        for p_idx in range(1, 5):
            sub_match = re.search(rf"#### 9\.5\.{p_idx+1}[\s\S]*?(?=#### 9\.5|\n---\n|$)", sec5_text)
            self.assertIsNotNone(sub_match, f"Could not extract Phase {p_idx} table")
            p_text = sub_match.group(0)
            for s_idx in range(1, 5):
                self.assertIn(f"**{s_idx}**", p_text, f"Missing step {s_idx} in Phase {p_idx} table")

        # ROE interlock traceability in SCN-05
        for roe in ["ROE-01", "ROE-02", "ROE-03", "ROE-04", "ROE-05", "ROE-06"]:
            self.assertIn(roe, sec5_text)

    def test_domain_agnosticism_and_zero_forbidden_nouns(self):
        """Verify 100% domain agnosticism with zero forbidden domain nouns."""
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, self.content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in 09_SCENARIOS_AND_TIMELINES.md: {violations}")

    def test_katex_and_markdown_table_integrity(self):
        """Verify zero KaTeX rendering violations and no LaTeX math delimiters inside table cells."""
        table_lines = [line for line in self.content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        findings = check_katex_text(self.content, source="09_SCENARIOS_AND_TIMELINES.md")
        self.assertEqual(
            findings,
            [],
            f"KaTeX / Markdown table integrity violations detected: {[str(f) for f in findings]}",
        )


if __name__ == "__main__":
    unittest.main()
