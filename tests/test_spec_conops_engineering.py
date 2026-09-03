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
CONOPS_01_OVERVIEW_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "01_METADATA_AND_OVERVIEW.md")
AGENTS_CONOPS_01_OVERVIEW_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "01_METADATA_AND_OVERVIEW.md")
MISSION_01_COMMANDERS_INTENT_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "mission_intent", "01_COMMANDERS_INTENT.md")
AGENTS_MISSION_01_COMMANDERS_INTENT_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "mission_intent", "01_COMMANDERS_INTENT.md")
CONOPS_02_DEFICIENCIES_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "02_DEFICIENCIES_AND_MOTIVATION.md")
AGENTS_CONOPS_02_DEFICIENCIES_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "02_DEFICIENCIES_AND_MOTIVATION.md")
CONOPS_04_USER_CLASSES_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "04_USER_CLASSES_AND_STAKEHOLDERS.md")
AGENTS_CONOPS_04_USER_CLASSES_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "04_USER_CLASSES_AND_STAKEHOLDERS.md")
CONOPS_08_ENVIRONMENTAL_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "08_ENVIRONMENTAL_MIL_STD_810H.md")
CONOPS_09_SCENARIOS_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "09_SCENARIOS_AND_TIMELINES.md")
AGENTS_CONOPS_09_SCENARIOS_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "09_SCENARIOS_AND_TIMELINES.md")
CONOPS_10_MAINTENANCE_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "10_MAINTENANCE_AND_GSE_SUPPORT.md")
AGENTS_CONOPS_10_MAINTENANCE_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "10_MAINTENANCE_AND_GSE_SUPPORT.md")
CONOPS_11_TRADE_STUDIES_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "11_IMPACTS_AND_TRADE_STUDIES.md")
AGENTS_CONOPS_11_TRADE_STUDIES_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "11_IMPACTS_AND_TRADE_STUDIES.md")
CONOPS_12_EMERGENCY_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "12_EMERGENCY_DECISION_MATRIX.md")
AGENTS_CONOPS_12_EMERGENCY_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops", "12_EMERGENCY_DECISION_MATRIX.md")
CONOPS_07_OPTX_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "conops", "07_OPTX_EXCHANGES.md")
MISSION_03_MATH_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units", "mission_intent", "03_INCOSE_MOE_MOP_MATH.md")
AGENTS_MISSION_03_MATH_PATH = os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units", "mission_intent", "03_INCOSE_MOE_MOP_MATH.md")





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
            "01_METADATA_AND_OVERVIEW.md",
            "02_DEFICIENCIES_AND_MOTIVATION.md",
            "03_PROPOSED_CAPABILITIES.md",
            "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            "05_AIRSPACE_AND_SORA_RISK.md",
            "06_UAF_OPERATIONAL_ACTIVITIES.md",
            "07_OPTX_EXCHANGES.md",
            "08_ENVIRONMENTAL_MIL_STD_810H.md",
            "09_SCENARIOS_AND_TIMELINES.md",
            "10_MAINTENANCE_AND_GSE_SUPPORT.md",
            "11_IMPACTS_AND_TRADE_STUDIES.md",
            "12_EMERGENCY_DECISION_MATRIX.md",
        ]
        for unit in conops_units:
            self.assertIn(unit, content, f"Missing ConOps unit '{unit}' in skill documentation")

        # 10 Mission Intent unit files
        mission_units = [
            "01_COMMANDERS_INTENT.md",
            "02_MISSION_ESSENTIAL_TASK_LIST.md",
            "03_INCOSE_MOE_MOP_MATH.md",
            "04_MULTI_DOMAIN_THREAT_MATRIX.md",
            "05_PACE_C2_PLAN.md",
            "06_ROE_SAFETY_INTERLOCKS.md",
            "07_AIRSPACE_GEOZONES.md",
            "08_GO_NO_GO_MATRIX.md",
            "09_BINGO_ENERGY_MATH.md",
            "10_OPERATIONAL_ALLOCATION_TAGS.md",
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

        # Modular Unit Storage canonical uppercase filenames
        self.assertIn("01_METADATA_AND_OVERVIEW.md", content)
        self.assertIn("12_EMERGENCY_DECISION_MATRIX.md", content)
        self.assertIn("01_COMMANDERS_INTENT.md", content)
        self.assertIn("10_OPERATIONAL_ALLOCATION_TAGS.md", content)

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
            CONOPS_01_OVERVIEW_PATH,
            CONOPS_04_USER_CLASSES_PATH,
            CONOPS_07_OPTX_PATH,
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


    def test_3_tier_safety_threat_derivation_lifecycle(self):
        """Verify formal documentation of the 3-Tier Multi-Run Safety & Threat Derivation Lifecycle."""
        for path in [CONOPS_SKILL_PATH, CONOPS_RULE_PATH]:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("The 3-Tier Multi-Run Safety & Threat Derivation Lifecycle", content)
            self.assertIn("Tier 1 (Run 1: Mission Intent / Concept Formulation)", content)
            self.assertIn("SAE ARP4761 §3", content)
            self.assertIn("MIL-STD-882E Task 202", content)
            self.assertIn("Tier 2 (Run 2: ConOps / Logical Architecture & SysML)", content)
            self.assertIn("MIL-STD-1629A Method 101", content)
            self.assertIn("Tier 3 (Run 3: Detailed Engineering & Physical BOM)", content)
            self.assertIn("MIL-STD-1629A Method 102", content)

            # Mission functions
            for fn in ["Propel", "Navigate", "Communicate", "Sense", "Contain"]:
                self.assertIn(fn, content)

            # Operating domains
            for dom in ["Kinetic", "Mechanical", "Power/Thermal", "Environmental", "EW", "Cyber", "Optical", "Signature", "Human Factors", "CBRN"]:
                self.assertIn(dom, content)

            # Hazard guide words
            for gw in ["Loss", "Degraded", "Intermittent", "Uncommanded"]:
                self.assertIn(gw, content)

            # Circular dependency deadlock clarification
            self.assertIn("circular dependency", content.lower())
            self.assertIn("piece-part bom fmeca", content.lower())

    def test_conops_01_metadata_and_overview_structure(self):
        """Verify 01_METADATA_AND_OVERVIEW.md populates Sections 1.3.1, 1.3.2, 1.3.3, 1.4, and 1.5 (#117, #123, #128, #121)."""
        self.assertTrue(os.path.isfile(CONOPS_01_OVERVIEW_PATH), f"Missing {CONOPS_01_OVERVIEW_PATH}")
        with open(CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Section 1.3.1: Parametric Coordinate Reference Frames Math
        self.assertIn("### 1.3.1 Parametric Coordinate Reference Frames Math", content)
        self.assertIn("Primary Global Geodetic Frame", content)
        self.assertIn("Local Tangent Plane / Navigation Frame", content)
        self.assertIn("Body-Fixed Geometric Frame", content)
        self.assertIn(r"\phi", content)
        self.assertIn(r"\lambda", content)
        self.assertIn(r"h_{\mathrm{ellips}}", content)
        self.assertIn("X_n, Y_n, Z_n", content)
        self.assertIn("X_b, Y_b, Z_b", content)
        self.assertIn(r"\boldsymbol{\omega}_{b/n}", content)
        self.assertIn(r"\mathbf{v}_b", content)

        # Section 1.3.2: Parametric Subsystem Mass/Resource Budget Breakdown Table
        self.assertIn("### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table", content)
        ast_groups = [
            "Airframe Structure",
            "Avionics & Processing",
            "Propulsion & Power Distribution",
            "Energy Storage Subsystem",
            "Primary Mission Payload",
            "Autonomous Failsafe Containment",
        ]
        for group in ast_groups:
            self.assertIn(group, content)
        self.assertIn("100.0% MTOW", content)

        # Section 1.3.3: Master Physical Limits Table
        self.assertIn("### 1.3.3 Master Physical Limits Table", content)
        physical_limits = [
            "Maximum Takeoff Weight (MTOW)",
            "Maximum Payload Mass Capacity",
            "Physical Dimensions",
            "Nominal Cruise Velocity",
            "Maximum Permissible Operating Velocity",
            "Minimum Controllable / Stall Velocity",
            "Maximum Operating Ceiling",
            "Command & Control (C2) Datalink Range",
            "Mission Operational Endurance",
            "Environmental Operating Temperature Envelope",
        ]
        for param in physical_limits:
            self.assertIn(param, content)

        # Section 1.4: Abstract UAF Context Diagram (Mermaid flowchart TB with 5 segments / 6 subgraphs across 15 interface links)
        self.assertIn("### 1.4 Abstract UAF Context Diagram", content)
        self.assertIn("flowchart TB", content)
        segments = [
            "SpaceSegment",
            "VehicleSegment",
            "ControlSegment",
            "SupportSegment",
            "OperationalRoles",
            "ExternalAuthorities",
        ]
        for seg in segments:
            self.assertIn(seg, content)

        # Section 1.5: Normative Standards & Regulatory Baseline Table (13 standards)
        self.assertIn("### 1.5 Normative Standards & Regulatory Baseline", content)
        standards = [
            "RTCA DO-178C (DAL-A)",
            "RTCA DO-254",
            "RTCA DO-365B",
            "RTCA DO-362A",
            "MIL-STD-882E",
            "MIL-STD-810H",
            "MIL-STD-461G",
            "ISO/IEC/IEEE 29148:2018",
            "ISO 26262:2018",
            "ASTM F3411-22a",
            "ASTM F3269-17",
            "JARUS SORA v2.5",
            "NIST SP 800-82r3",
        ]
        for std in standards:
            self.assertIn(std, content)

        # Issue Tracking references
        self.assertIn("Fixes #117, #123, #128, #121", content)

    def test_conops_11_impacts_and_trade_studies_structure(self):
        """Verify 11_IMPACTS_AND_TRADE_STUDIES.md populates Trade Studies 1, 2, and 3 with full Pugh matrices, sensitivity equations, parameter tables, and domain agnosticism (#118, #129, #132)."""
        self.assertTrue(os.path.isfile(CONOPS_11_TRADE_STUDIES_PATH), f"Missing {CONOPS_11_TRADE_STUDIES_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_11_TRADE_STUDIES_PATH), f"Missing {AGENTS_CONOPS_11_TRADE_STUDIES_PATH}")

        with open(CONOPS_11_TRADE_STUDIES_PATH, "r", encoding="utf-8") as f:
            content = f.read()
        with open(AGENTS_CONOPS_11_TRADE_STUDIES_PATH, "r", encoding="utf-8") as f:
            mirror_content = f.read()

        self.assertEqual(content, mirror_content, "Mirror mismatch between skills/ and .agents/ for 11_IMPACTS_AND_TRADE_STUDIES.md")

        # Heading presence
        self.assertIn("## 11. Operational Impacts, System Limitations & Documented Trade Studies", content)
        self.assertIn("### 11.1 Operational Workflow & Deployment Impacts", content)
        self.assertIn("### 11.2 Organizational Roles & Training Impacts", content)
        self.assertIn("### 11.3 System Limitations & Operational Boundaries", content)
        self.assertIn("### 11.4 Documented Engineering Trade Studies", content)

        # Traceability references
        self.assertIn("Fixes #118, #129, #132", content)

        # Trade Study 1: Energy & Power Storage Architecture (Fix #118)
        self.assertIn("#### 11.4.1 Trade Study 1: Energy & Power Storage Architecture", content)
        self.assertIn("Multi-Criteria Pugh Decision Matrix (Trade Study 1)", content)
        self.assertIn("High-Capacity Cylindrical Chemistry with Integrated Thermal Heating", content)
        self.assertIn("Mathematical Sensitivity Analysis Formulation (Trade Study 1)", content)
        self.assertIn(r"S_j(w_{\mathrm{cold}}) &= \sum_{i \neq \mathrm{cold}} w_i s_{ij} + w_{\mathrm{cold}} s_{\mathrm{cold}, j}", content)

        # Trade Study 2: Edge Neural Compute vs Datalink Compression (Fix #129)
        self.assertIn("#### 11.4.2 Trade Study 2: Edge Neural Compute vs Datalink Compression", content)
        self.assertIn("Multi-Criteria Pugh Decision Matrix (Trade Study 2)", content)
        self.assertIn("Onboard Edge Neural Inference Accelerator with Dynamic Context Compression", content)
        self.assertIn("Mathematical Sensitivity Analysis Formulation (Trade Study 2)", content)
        self.assertIn(r"S_j(P_{\mathrm{jam}}) &= \sum_{i \neq \mathrm{jam}} w_i s_{ij} + w_{\mathrm{jam}} s_{\mathrm{jam}, j}(P_{\mathrm{jam}})", content)

        # Trade Study 3: Autonomous Failsafe Containment vs Actuator Redundancy (Fix #132)
        self.assertIn("#### 11.4.3 Trade Study 3: Autonomous Failsafe Containment vs Actuator Redundancy", content)
        self.assertIn("Multi-Criteria Pugh Decision Matrix (Trade Study 3)", content)
        self.assertIn("Integrated Autonomous Failsafe Containment Subsystem", content)
        self.assertIn("Mathematical Sensitivity Analysis Formulation (Trade Study 3)", content)
        self.assertIn(r"E_k(m) &= \frac{1}{2} m v_{\mathrm{term}}^2(m) = \frac{m^2 g}{\rho S C_d}", content)

        # Parameter Tables ("- Parameter Definitions & Engineering Units:")
        self.assertEqual(content.count("- Parameter Definitions & Engineering Units:"), 3)

        # KaTeX Rendering Integrity check
        cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
        cleaned = re.sub(r"`+.*?`+", "", cleaned)

        # Balanced $$ delimiters
        parts = cleaned.split("$$")
        self.assertEqual(
            (len(parts) - 1) % 2,
            0,
            f"Unbalanced $$ delimiters in 11_IMPACTS_AND_TRADE_STUDIES.md (found {len(parts) - 1} delimiters)",
        )

        # Balanced \begin{aligned} and \end{aligned}
        num_begin_aligned = len(re.findall(r"\\begin\{aligned\}", cleaned))
        num_end_aligned = len(re.findall(r"\\end\{aligned\}", cleaned))
        self.assertEqual(num_begin_aligned, num_end_aligned)
        self.assertGreaterEqual(num_begin_aligned, 3)

        # No forbidden \begin{align}
        self.assertFalse(re.search(r"\\begin\{align\*?\}", cleaned))

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        # Domain Agnosticism & Parametric Placeholders check
        from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")
        self.assertEqual(violations, [], f"Forbidden domain nouns found in 11_IMPACTS_AND_TRADE_STUDIES.md: {violations}")

        # Ensure parametric placeholders are present
        placeholders = re.findall(r"\{\{[A-Za-z0-9_]+\}\}", content)
        self.assertGreaterEqual(len(placeholders), 10, f"Expected >= 10 parametric placeholders, found {len(placeholders)}")

    def test_conops_02_deficiencies_and_motivation_structure(self):
        """Verify 02_DEFICIENCIES_AND_MOTIVATION.md populates Section 2.2 GAP-01..GAP-04 table and Section 2.3 3-Domain Legacy Deficiencies Matrix (#139)."""
        self.assertTrue(os.path.isfile(CONOPS_02_DEFICIENCIES_PATH), f"Missing {CONOPS_02_DEFICIENCIES_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_02_DEFICIENCIES_PATH), f"Missing {AGENTS_CONOPS_02_DEFICIENCIES_PATH}")

        with open(CONOPS_02_DEFICIENCIES_PATH, "r", encoding="utf-8") as f:
            content = f.read()
        with open(AGENTS_CONOPS_02_DEFICIENCIES_PATH, "r", encoding="utf-8") as f:
            mirror_content = f.read()

        self.assertEqual(content, mirror_content, "Mirror mismatch between skills/ and .agents/ for 02_DEFICIENCIES_AND_MOTIVATION.md")

        # Headings presence
        self.assertIn("## 2. Current Situation, Deficiency Analysis & Operational Motivation", content)
        self.assertIn("### 2.1 Current Operational Baseline (Predecessors)", content)
        self.assertIn("### 2.2 Operational Capability Gaps", content)
        self.assertIn("### 2.3 3-Domain Legacy Deficiencies Matrix", content)
        self.assertIn("#### 2.3.1 Technical Deficiencies Detail", content)
        self.assertIn("#### 2.3.2 Operational Deficiencies Detail", content)
        self.assertIn("#### 2.3.3 Human Factors Deficiencies Detail", content)
        self.assertIn("### 2.4 Mission Drivers & User Operational Problems", content)

        # Section 2.2: Capability Gaps GAP-01 through GAP-04
        gaps = ["GAP-01", "GAP-02", "GAP-03", "GAP-04"]
        for gap in gaps:
            self.assertIn(f"**{gap}**", content)
            self.assertIn(f"/// OperationalAllocation: [{gap}]", content)

        # Section 2.2 Table Columns: Operational Manifestation, Technical Root Cause, Mission Impact, Severity, Gate 24 Allocation Tag
        gap_columns = [
            "Gap ID",
            "Capability Gap Name",
            "Operational Manifestation",
            "Technical Root Cause",
            "Mission Impact & Risk",
            "Severity",
            "Gate 24 Allocation Tag",
        ]
        for col in gap_columns:
            self.assertIn(col, content)

        # Section 2.3: 3-Domain Legacy Deficiencies Matrix
        domains = ["Technical Deficiencies", "Operational Deficiencies", "Human Factors Deficiencies"]
        for domain in domains:
            self.assertIn(f"**{domain}**", content)

        matrix_columns = [
            "Deficiency Domain",
            "Deficiency ID",
            "Legacy Baseline Deficiency Description",
            "Quantified Baseline Limitation",
            "Safety Risk Categorization",
            "Proposed Architectural Mitigation",
        ]
        for col in matrix_columns:
            self.assertIn(col, content)

        # Specific deficiency IDs
        def_ids = [
            "DEF-TECH-01", "DEF-TECH-02", "DEF-TECH-03", "DEF-TECH-04",
            "DEF-OPS-01", "DEF-OPS-02", "DEF-OPS-03", "DEF-OPS-04",
            "DEF-HUM-01", "DEF-HUM-02", "DEF-HUM-03",
        ]
        for did in def_ids:
            self.assertIn(f"`{did}`", content)

        # Quantified baseline limitations
        quant_metrics = [
            "MTBF_baseline <= MTBF_threshold",
            "tau_loop > tau_stability_limit",
            "{{INGRESS_PROTECTION_RATING}}",
            "t_collapse < 10 ms",
            "R_margin < R_buffer_req",
            "t_coord > 300 s",
            "E_reserve < 0.20 * E_capacity",
            "t_turnaround_legacy >> t_turnaround_target",
            "TLX_manual > TLX_threshold",
            "tau_human > tau_containment_req",
            "t_shift > t_shift_max",
        ]
        for qm in quant_metrics:
            self.assertIn(qm, content)

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        # Domain Agnosticism check
        from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")
        self.assertEqual(violations, [], f"Forbidden domain nouns found in 02_DEFICIENCIES_AND_MOTIVATION.md: {violations}")

    def test_conops_08_environmental_mil_std_810h_structure(self):
        """Verify 08_ENVIRONMENTAL_MIL_STD_810H.md populates Section 8.1 12-method table and Section 8.2.1-8.2.12 (#127, #134, #137)."""
        self.assertTrue(os.path.isfile(CONOPS_08_ENVIRONMENTAL_PATH), f"Missing {CONOPS_08_ENVIRONMENTAL_PATH}")
        with open(CONOPS_08_ENVIRONMENTAL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Headings presence
        self.assertIn("## 8. Operational Environments & MIL-STD-810H Environmental Stress Qualification", content)
        self.assertIn("### 8.1 Master 12-Method Environmental Stress Qualification Table", content)
        self.assertIn("### 8.2 Granular Test Method Breakdowns", content)
        self.assertIn("### 8.3 Ingress Protection (IEC 60529) & Environmental Sealing Architecture", content)
        self.assertIn("### 8.4 Electromagnetic Compatibility (EMC/EMI) & RF Environments", content)
        self.assertIn("### 8.5 Physical Spatial Constraints & Deployment Envelopes", content)

        # 12 canonical methods in Section 8.1 table
        methods = [
            "M-500.6", "M-501.7", "M-502.7", "M-503.7",
            "M-505.7", "M-506.6", "M-507.6", "M-509.7",
            "M-510.7", "M-514.8", "M-516.8", "M-521.4",
        ]
        for m in methods:
            self.assertIn(f"**{m}**", content)

        # 12 subsections in 8.2
        for idx in range(1, 13):
            self.assertIn(f"#### 8.2.{idx}", content)

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        # Domain Agnosticism check
        from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")
        self.assertEqual(violations, [], f"Forbidden domain nouns found in 08_ENVIRONMENTAL_MIL_STD_810H.md: {violations}")

        # Parametric placeholders check
        placeholders = re.findall(r"\{\{[A-Za-z0-9_]+\}\}", content)
        self.assertGreaterEqual(len(placeholders), 30, f"Expected >= 30 parametric placeholders, found {len(placeholders)}")

    def test_conops_09_scenarios_and_timelines_structure(self):
        """Verify 09_SCENARIOS_AND_TIMELINES.md populates SCN-01 (8-step), SCN-02 (6-step), SCN-03 (6-step), SCN-04 (6-step), and SCN-05 (stateDiagram-v2 + 4 phase tables) (Fixes #126, #131, #138)."""
        self.assertTrue(os.path.isfile(CONOPS_09_SCENARIOS_PATH), f"Missing {CONOPS_09_SCENARIOS_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_09_SCENARIOS_PATH), f"Missing {AGENTS_CONOPS_09_SCENARIOS_PATH}")

        with open(CONOPS_09_SCENARIOS_PATH, "r", encoding="utf-8") as f:
            content = f.read()
        with open(AGENTS_CONOPS_09_SCENARIOS_PATH, "r", encoding="utf-8") as f:
            mirror_content = f.read()

        self.assertEqual(content, mirror_content, "Mirror mismatch between skills/ and .agents/ for 09_SCENARIOS_AND_TIMELINES.md")

        # Headings presence
        self.assertIn("## 9. Multi-Threaded Operational Scenarios & System Timelines", content)
        self.assertIn("### 9.1 Scenario SCN-01: Nominal Lifecycle Thread", content)
        self.assertIn("### 9.2 Scenario SCN-02: High-Throughput State Tracking & Target Processing", content)
        self.assertIn("### 9.3 Scenario SCN-03: Degraded C2 Lost-Link & Autonomous Fallback Return", content)
        self.assertIn("### 9.4 Scenario SCN-04: Dynamic Geofence Boundary Divert", content)
        self.assertIn("### 9.5 Scenario SCN-05: Controlled Safety Interlock Action", content)
        self.assertIn("#### 9.5.1 Safety Interlock State Machine & Deterministic Transitions", content)
        self.assertIn("#### 9.5.2 Phase 1: Ingress & Station Keeping Execution Verification", content)
        self.assertIn("#### 9.5.3 Phase 2: Positive Identification & Interlock Check Execution Verification", content)
        self.assertIn("#### 9.5.4 Phase 3: Dual-Consent Arming & Execution Verification", content)
        self.assertIn("#### 9.5.5 Phase 4: Post-Action Assessment & Telemetry Dump Execution Verification", content)

        # Mandatory columns in timeline tables
        timeline_cols = [
            "Step Number",
            "Elapsed Time (T+)",
            "Stimulus / Trigger",
            "Actor / Performer",
            "Action Executed",
            "Telemetry Stream",
            "Decision Gate",
            "Exception Branch",
            "Exit Criterion",
        ]
        for col in timeline_cols:
            self.assertIn(col, content)

        # Scenario 1 (SCN-01): Full 8 steps
        sec1_match = re.search(r'### 9\.1 Scenario SCN-01[\s\S]*?(?=### 9\.2)', content)
        self.assertIsNotNone(sec1_match)
        sec1_text = sec1_match.group(0)
        for step_idx in range(1, 9):
            self.assertIn(f"**{step_idx}**", sec1_text)

        # Scenario 2 (SCN-02): Full 6 steps
        sec2_match = re.search(r'### 9\.2 Scenario SCN-02[\s\S]*?(?=### 9\.3)', content)
        self.assertIsNotNone(sec2_match)
        sec2_text = sec2_match.group(0)
        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec2_text)

        # Scenario 3 (SCN-03): Full 6 steps
        sec3_match = re.search(r'### 9\.3 Scenario SCN-03[\s\S]*?(?=### 9\.4)', content)
        self.assertIsNotNone(sec3_match)
        sec3_text = sec3_match.group(0)
        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec3_text)

        # Scenario 4 (SCN-04): Full 6 steps
        sec4_match = re.search(r'### 9\.4 Scenario SCN-04[\s\S]*?(?=### 9\.5)', content)
        self.assertIsNotNone(sec4_match)
        sec4_text = sec4_match.group(0)
        for step_idx in range(1, 7):
            self.assertIn(f"**{step_idx}**", sec4_text)

        # Scenario 5 (SCN-05): Mermaid stateDiagram-v2 state machine
        sec5_match = re.search(r'### 9\.5 Scenario SCN-05[\s\S]*$', content)
        self.assertIsNotNone(sec5_match)
        sec5_text = sec5_match.group(0)
        self.assertIn("stateDiagram-v2", sec5_text)
        self.assertIn("Phase_Ingress_StationKeeping", sec5_text)
        self.assertIn("Phase_PID_InterlockCheck", sec5_text)
        self.assertIn("Phase_DualConsent_ArmingExecution", sec5_text)
        self.assertIn("Phase_PostAction_AssessmentDump", sec5_text)

        # Scenario 5: 4 decomposed phase execution verification tables
        phase_headings = [
            "#### 9.5.2 Phase 1: Ingress & Station Keeping Execution Verification",
            "#### 9.5.3 Phase 2: Positive Identification & Interlock Check Execution Verification",
            "#### 9.5.4 Phase 3: Dual-Consent Arming & Execution Verification",
            "#### 9.5.5 Phase 4: Post-Action Assessment & Telemetry Dump Execution Verification",
        ]
        for ph in phase_headings:
            self.assertIn(ph, sec5_text)

        # Traceability tokens
        for oa in ["OA-01", "OA-02", "OA-03", "OA-04", "OA-05", "OA-06", "OA-07", "OA-08"]:
            self.assertIn(oa, content)
        for optx in ["OpTx-01", "OpTx-02", "OpTx-03", "OpTx-04", "OpTx-05", "OpTx-06", "OpTx-07", "OpTx-08"]:
            self.assertIn(optx, content)
        for emg in ["EMG-01", "EMG-02", "EMG-03", "EMG-05", "EMG-06", "EMG-07"]:
            self.assertIn(emg, content)
        for roe in ["ROE-01", "ROE-02", "ROE-03", "ROE-04", "ROE-05", "ROE-06"]:
            self.assertIn(roe, content)

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        # Domain Agnosticism check
        from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")
        self.assertEqual(violations, [], f"Forbidden domain nouns found in 09_SCENARIOS_AND_TIMELINES.md: {violations}")

    def test_conops_04_user_classes_and_stakeholders_structure(self):
        """Verify 04_USER_CLASSES_AND_STAKEHOLDERS.md populates Section 4.4 NASA-TLX matrix, Section 4.4.1 shift rotation, Section 4.5.1 4-way handoff diagram, and Section 4.5.2 timeout & rejection protocol (Fixes #120, #119)."""
        self.assertTrue(os.path.isfile(CONOPS_04_USER_CLASSES_PATH), f"Missing {CONOPS_04_USER_CLASSES_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_04_USER_CLASSES_PATH), f"Missing {AGENTS_CONOPS_04_USER_CLASSES_PATH}")

        with open(CONOPS_04_USER_CLASSES_PATH, "r", encoding="utf-8") as f:
            content = f.read()
        with open(AGENTS_CONOPS_04_USER_CLASSES_PATH, "r", encoding="utf-8") as f:
            mirror_content = f.read()

        self.assertEqual(content, mirror_content, "Mirror mismatch between skills/ and .agents/ for 04_USER_CLASSES_AND_STAKEHOLDERS.md")

        # Headings presence
        self.assertIn("## 4. User Classes, Stakeholder Taxonomy & Operational Lifecycle Modes", content)
        self.assertIn("### 4.1 Stakeholder Roster", content)
        self.assertIn("### 4.2 User Class Taxonomy", content)
        self.assertIn("### 4.3 Skill Prerequisites & Minimum Qualifications", content)
        self.assertIn("### 4.4 Workload Constraints & Human Factors Considerations", content)
        self.assertIn("### 4.4.1 Operational Shift Rotation Protocol", content)
        self.assertIn("### 4.5 Authority Handoff Chains & Control Transfer Protocols", content)
        self.assertIn("### 4.5.1 Abstract Cryptographic 4-Way Control Handoff Sequence Diagram", content)
        self.assertIn("### 4.5.2 Timeout & Rejection Protocol", content)
        self.assertIn("### 4.6 Operational Lifecycle Stages", content)

        # Traceability references
        self.assertIn("Fixes #120, #119", content)

        # User classes UC-01 to UC-05
        for uc in ["UC-01", "UC-02", "UC-03", "UC-04", "UC-05"]:
            self.assertIn(f"**{uc}**", content)

        # Section 4.4: 6-Dimensional NASA-TLX Table
        tlx_dims = [
            "Mental Demand",
            "Physical Demand",
            "Temporal Demand",
            "Performance",
            "Effort",
            "Frustration",
        ]
        for dim in tlx_dims:
            self.assertIn(dim, content)

        self.assertIn("Score <= 35", content)
        self.assertIn("Score <= 55", content)
        self.assertIn("TLX_nominal_max <= 35", content)
        self.assertIn("TLX_contingency_max <= 55", content)

        # Section 4.4.1: Operational Shift Rotation Protocol
        self.assertIn("t_shift <= 4.0 hr", content)
        self.assertIn("t_rest >= 30.0 min", content)
        self.assertIn("t_daily_max <= 8.0 hr", content)

        # Section 4.5.1: Sequence diagram with 4 participants across 9 steps
        self.assertIn("sequenceDiagram", content)
        participants = ["PrimaryConsole", "VehicleController", "SecondaryConsole", "CryptographicAuthService"]
        for p in participants:
            self.assertIn(p, content)

        for step in [
            "1. Request Handoff Token",
            "2. Issue Signed Token",
            "3. Transmit Control Request",
            "4. Validate Token",
            "5. Token Verification Response",
            "6. Command Control Relinquishment",
            "7. Acknowledge Relinquish",
            "8. Grant Active C2 Authority",
            "9. Confirm Active C2",
        ]:
            self.assertIn(step, content)

        # Section 4.5.2: Bounded Timeout Recovery (tau_timeout = 5.0 s) & 4 Rejection Criteria
        self.assertIn("5.0", content)
        for rej in ["REJ-01", "REJ-02", "REJ-03", "REJ-04"]:
            self.assertIn(f"**{rej}**", content)

        # Markdown Table Math Prohibition: No $ in table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|")]
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

        # Domain Agnosticism check
        from tests.test_canonical_templates import FORBIDDEN_DOMAIN_NOUNS
        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")
        self.assertEqual(violations, [], f"Forbidden domain nouns found in 04_USER_CLASSES_AND_STAKEHOLDERS.md: {violations}")

        # Parametric placeholders check
        placeholders = re.findall(r"\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}", content)
        self.assertGreaterEqual(len(placeholders), 5, f"Expected >= 5 parametric placeholders, found {len(placeholders)}")

    def test_section_1_3_system_boundary_math_and_table(self):
        """Verify Section 1.3 in 01_METADATA_AND_OVERVIEW.md has valid KaTeX math formatting and formal Parameter Definitions table with plain text symbols (Fixes #151, #155)."""
        self.assertTrue(os.path.isfile(CONOPS_01_OVERVIEW_PATH), f"Missing {CONOPS_01_OVERVIEW_PATH}")
        with open(CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Section 1.3 header
        self.assertIn("### 1.3 System Boundary & Operational State Space", content)

        # KaTeX math formatting in Section 1.3 prose
        self.assertIn(r"$\Omega_{\mathrm{state}} \subset \mathbb{R}^n$", content)
        self.assertIn(r"$\mathbf{X}_{\mathrm{boundary}} = [\mathbf{x}_{\mathrm{min}}, \mathbf{x}_{\mathrm{max}}]^\top$", content)
        self.assertIn(r"($R_{\mathrm{buffer}}$)", content)
        self.assertIn(r"$\text{Range}_{\mathrm{max}}(\text{Link}_{\mathrm{C2}})$", content)

        # Operational State Space Parameter Definitions & Engineering Units table
        self.assertIn("- **Operational State Space Parameter Definitions & Engineering Units:**", content)
        self.assertIn("| Symbol / Parameter | Domain / Context | Description | Dimension / Limits | Engineering Unit | Normative / Safety Basis |", content)
        self.assertIn(r"| Ω_state | State Space Domain | Admissible operational state space envelope (Ω_state ⊂ R^n) | Compact subset of R^n (n >= 6) | Dimensionless | ISO/IEC/IEEE 29148:2018 §6.4.2 |", content)
        self.assertIn(r"| X_boundary | State Vector Bounds | Bounding box of admissible vehicle operational states [x_min, x_max]^T | Bounded hyper-rectangle | Mixed SI Units | ASTM F3269-17 §6.2 |", content)
        self.assertIn(r"| x_min | State Lower Limit | Minimum permissible state vector threshold | [phi_min, lambda_min, h_min, u_min, v_min, w_min]^T | rad, rad, m, m/s | SORA Annex B M1 Mitigations |", content)
        self.assertIn(r"| x_max | State Upper Limit | Maximum permissible state vector threshold | [phi_max, lambda_max, h_max, u_max, v_max, w_max]^T | rad, rad, m/s | SORA Annex B M1 Mitigations |", content)
        self.assertIn(r"| R_buffer | Spatial Containment | Verified 1:1 parametric lateral containment safety buffer radius | R_buffer >= 1.0 * Distance_containment | m | JARUS SORA v2.5 Step #2 |", content)
        self.assertIn(r"| Range_max(Link_C2) | C2 Comms Margin | Maximum certified C2 data link operational range | Range_max >= Range_nominal | km | RTCA DO-362A §2.2.1 |", content)
        self.assertIn(r"| tau_containment | Emergency Response | Maximum allowable failsafe containment response time | tau_containment <= 2.0 | s | ASTM F3269-17 §7.1 |", content)

        # Markdown Table Math Prohibition: No $ in Section 1.3 table lines
        table_lines = [line for line in content.splitlines() if line.startswith("|") and any(sym in line for sym in ["Ω_state", "X_boundary", "x_min", "x_max", "R_buffer", "Range_max(Link_C2)", "tau_containment"])]
        self.assertGreaterEqual(len(table_lines), 7, f"Expected >= 7 table lines, found {len(table_lines)}")
        for line in table_lines:
            self.assertNotIn("$", line, f"Found LaTeX math delimiter '$' in table line: {line}")

    def test_incose_moe_mop_math_operational_availability_tokens(self):
        """Verify 03_INCOSE_MOE_MOP_MATH.md uses resolved symbolic identifiers A_o_threshold and A_o_objective instead of raw template tokens (Fixes #154)."""
        self.assertTrue(os.path.isfile(MISSION_03_MATH_PATH), f"Missing {MISSION_03_MATH_PATH}")
        with open(MISSION_03_MATH_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Check line 106 MoE-01 row
        self.assertIn("| MoE-01 | MoE | Operational Availability | Ao = MTBM / (MTBM + MDT) | A_o_threshold | A_o_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |", content)
        self.assertNotIn("{{OPERATIONAL_AVAILABILITY_THRESHOLD}}", content)
        self.assertNotIn("{{OPERATIONAL_AVAILABILITY_OBJECTIVE}}", content)


    def test_section_7_3_optx_katex_math_notation(self):
        """Verify Section 7.3 in 07_OPTX_EXCHANGES.md has valid KaTeX math formatting without unbracketed double subscripts (Fixes #152)."""
        self.assertTrue(os.path.isfile(CONOPS_07_OPTX_PATH), f"Missing {CONOPS_07_OPTX_PATH}")
        with open(CONOPS_07_OPTX_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Section 7.3 header
        self.assertIn("### 7.3 Avionic Network Quality of Service (QoS) Stack Allocation", content)

        # KaTeX math formatting in Section 7.3 formulas
        self.assertIn(r"\text{Util}_{\text{bus}} &= \sum_{i=1}^{N_{\text{bus}}} \frac{C_i}{T_i} \le \text{Util}_{\text{bus\_max}}", content)
        self.assertIn(r"$\text{Util}_{\text{bus}}$: Total deterministic real-time bus utilization under worst-case burst conditions.", content)
        self.assertIn(r"$N_{\text{bus}}$: Total number of active periodic message streams", content)
        self.assertIn(r"$T_i = 1 / f_{\text{rate}, i}$", content)
        self.assertIn(r"$\text{Util}_{\text{bus\_max}}$: Maximum allowable bus utilization ceiling", content)
        self.assertIn(r"$\text{Util}_{\text{bus\_max}} \le 0.60$", content)
        self.assertIn(r"$t_{\text{failover}} \le \tau_{\text{bus\_failover\_max}}$", content)
        self.assertIn(r"$\text{Throughput}_{\text{payload\_bus}} \ge 1.0\text{ Gbps}$", content)
        self.assertIn(r"\tau_{\text{Primary\_max}}", content)
        self.assertIn(r"\tau_{\text{Alternate\_max}}", content)
        self.assertIn(r"\tau_{\text{Contingency\_max}}", content)
        self.assertIn(r"\tau_{\text{Emergency\_max}}", content)
        self.assertIn(r"$f_{\text{remote\_id\_rate}} = 1.0\text{ Hz}$ to $2.0\text{ Hz}$", content)
        self.assertIn(r"\tau_{\text{remote\_id}} \le 200\text{ ms}", content)
        self.assertIn(r"$t_{\text{actuate}} \le \tau_{\text{squib\_latency\_max}} \le 10\text{ ms}$", content)

        # Ensure no invalid \mathrm subscript notations with raw/escaped underscores exist in Section 7.3
        self.assertNotIn(r"\mathrm{bus", content)
        self.assertNotIn(r"\mathrm{Primary", content)
        self.assertNotIn(r"\mathrm{Alternate", content)
        self.assertNotIn(r"\mathrm{Contingency", content)
        self.assertNotIn(r"\mathrm{Emergency", content)
        self.assertNotIn(r"\mathrm{remote", content)
        self.assertNotIn(r"\mathrm{squib", content)
        self.assertNotIn(r"\mathrm{payload", content)

    def test_conops_units_10_11_12_katex_math_subscript_notation(self):
        """Verify Units 10, 11, and 12 contain zero \\mathrm{...\\_...} math subscripts and use \\text{...\\_...} (Fixes #156)."""
        paths_to_verify = [
            (CONOPS_10_MAINTENANCE_PATH, AGENTS_CONOPS_10_MAINTENANCE_PATH, "10_MAINTENANCE_AND_GSE_SUPPORT.md"),
            (CONOPS_11_TRADE_STUDIES_PATH, AGENTS_CONOPS_11_TRADE_STUDIES_PATH, "11_IMPACTS_AND_TRADE_STUDIES.md"),
            (CONOPS_12_EMERGENCY_PATH, AGENTS_CONOPS_12_EMERGENCY_PATH, "12_EMERGENCY_DECISION_MATRIX.md"),
        ]

        for primary_path, mirror_path, unit_name in paths_to_verify:
            self.assertTrue(os.path.isfile(primary_path), f"Missing {primary_path}")
            self.assertTrue(os.path.isfile(mirror_path), f"Missing {mirror_path}")
            with open(primary_path, "r", encoding="utf-8") as f:
                content = f.read()
            with open(mirror_path, "r", encoding="utf-8") as f:
                mirror_content = f.read()
            self.assertEqual(content, mirror_content, f"Mirror mismatch between skills/ and .agents/ for {unit_name}")

            # Verify zero \mathrm{..._...} snake_case subscripts in math mode
            self.assertFalse(
                re.search(r"\\mathrm\{[^}]*_[^}]*\}", content),
                f"Found invalid \\mathrm snake_case subscript with underscore in {unit_name}",
            )

        # Unit 10 specific \text{...} assertions
        with open(CONOPS_10_MAINTENANCE_PATH, "r", encoding="utf-8") as f:
            c10 = f.read()
        self.assertIn(r"\tau_{\text{PBIT\_max}}", c10)
        self.assertIn(r"$t_{\text{swap}} \le \tau_{\text{swap\_battery}}", c10)
        self.assertIn(r"$t_{\text{LRU\_swap}} \le \tau_{\text{swap\_LRU}}", c10)
        self.assertIn(r"\tau_{\text{turnaround\_max}}", c10)
        self.assertIn(r"\tau_{\text{swap\_FC}}", c10)
        self.assertIn(r"\tau_{\text{swap\_payload}}", c10)
        self.assertIn(r"\tau_{\text{swap\_actuator}}", c10)
        self.assertIn(r"T_{\text{op\_min}}", c10)
        self.assertIn(r"T_{\text{op\_max}}", c10)
        self.assertIn(r"\text{RH}_{\text{storage\_max}}", c10)
        self.assertIn(r"t_{\text{spares\_endurance}}", c10)

        # Unit 11 specific \text{...} assertions
        with open(CONOPS_11_TRADE_STUDIES_PATH, "r", encoding="utf-8") as f:
            c11 = f.read()
        self.assertIn(r"\tau_{\text{prep\_target}}", c11)
        self.assertIn(r"\tau_{\text{turnaround\_target}}", c11)
        self.assertIn(r"x_{\text{operating\_max}}", c11)
        self.assertIn(r"t_{\text{endurance\_nominal}}", c11)
        self.assertIn(r"t_{\text{endurance\_cold}}", c11)
        self.assertIn(r"m_{\text{payload\_max}}", c11)
        self.assertIn(r"m_{\text{system\_max}}", c11)
        self.assertIn(r"a_{\text{dist\_limit}}", c11)
        self.assertIn(r"R_{\text{precip\_max}}", c11)
        self.assertIn(r"\tau_{\text{deploy\_max}}", c11)

        # Unit 12 specific \text{...} assertions
        with open(CONOPS_12_EMERGENCY_PATH, "r", encoding="utf-8") as f:
            c12 = f.read()
        self.assertIn(r"\tau_{\text{deadline\_abort}}", c12)

    def test_operational_purpose_and_mission_template_tokens(self):
        """Verify 01_COMMANDERS_INTENT.md and 01_METADATA_AND_OVERVIEW.md use template tokens {{OPERATIONAL_PURPOSE}} and {{PRIMARY_OPERATIONAL_MISSION}} (Fixes #157)."""
        self.assertTrue(os.path.isfile(MISSION_01_COMMANDERS_INTENT_PATH), f"Missing {MISSION_01_COMMANDERS_INTENT_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_MISSION_01_COMMANDERS_INTENT_PATH), f"Missing {AGENTS_MISSION_01_COMMANDERS_INTENT_PATH}")
        self.assertTrue(os.path.isfile(CONOPS_01_OVERVIEW_PATH), f"Missing {CONOPS_01_OVERVIEW_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_01_OVERVIEW_PATH), f"Missing {AGENTS_CONOPS_01_OVERVIEW_PATH}")

        # Check Mission Intent 01_COMMANDERS_INTENT.md
        with open(MISSION_01_COMMANDERS_INTENT_PATH, "r", encoding="utf-8") as f:
            mission_c = f.read()
        with open(AGENTS_MISSION_01_COMMANDERS_INTENT_PATH, "r", encoding="utf-8") as f:
            agents_mission_c = f.read()
        self.assertEqual(mission_c, agents_mission_c, "Mirror mismatch between skills/ and .agents/ for 01_COMMANDERS_INTENT.md")
        self.assertIn("- **Operational Purpose:** {{OPERATIONAL_PURPOSE}}", mission_c)
        self.assertNotIn("The primary operational purpose of the tactical autonomous cyber-physical system is to execute persistent", mission_c)

        # Check ConOps 01_METADATA_AND_OVERVIEW.md
        with open(CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            conops_c = f.read()
        with open(AGENTS_CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            agents_conops_c = f.read()
        self.assertEqual(conops_c, agents_conops_c, "Mirror mismatch between skills/ and .agents/ for 01_METADATA_AND_OVERVIEW.md")
        self.assertIn("- **Primary Operational Mission:** {{PRIMARY_OPERATIONAL_MISSION}}", conops_c)
        self.assertNotIn("The system is engineered to execute autonomous closed-loop state trajectory execution", conops_c)

    def test_core_mission_capabilities_template_token(self):
        """Verify 01_METADATA_AND_OVERVIEW.md uses template token {{CORE_MISSION_CAPABILITIES}} under Core Mission Capabilities (Fixes #158)."""
        self.assertTrue(os.path.isfile(CONOPS_01_OVERVIEW_PATH), f"Missing {CONOPS_01_OVERVIEW_PATH}")
        self.assertTrue(os.path.isfile(AGENTS_CONOPS_01_OVERVIEW_PATH), f"Missing {AGENTS_CONOPS_01_OVERVIEW_PATH}")

        # Check ConOps 01_METADATA_AND_OVERVIEW.md
        with open(CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            conops_c = f.read()
        with open(AGENTS_CONOPS_01_OVERVIEW_PATH, "r", encoding="utf-8") as f:
            agents_conops_c = f.read()
        self.assertEqual(conops_c, agents_conops_c, "Mirror mismatch between skills/ and .agents/ for 01_METADATA_AND_OVERVIEW.md")
        self.assertIn("- **Core Mission Capabilities:**\n{{CORE_MISSION_CAPABILITIES}}", conops_c)
        self.assertNotIn("1. Autonomous closed-loop state trajectory tracking, corridor execution, and stationary state holding", conops_c)
        self.assertNotIn("3. Real-time high-throughput telemetry streaming and edge neural state inference processing.", conops_c)

    def test_no_mathrm_with_underscores_in_units(self):
        """Verify zero occurrences of \\mathrm{..._...} math mode notation exist in modular unit markdown files (Fixes #160)."""
        pattern = re.compile(r"\\mathrm\{[^}]*_[^}]*\}")
        unit_dirs = [
            os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units"),
            os.path.join(REPO_ROOT, ".agents", "skills", "spec-conops-engineering", "resources", "units"),
        ]
        found_violations = []
        for udir in unit_dirs:
            self.assertTrue(os.path.isdir(udir), f"Missing units directory {udir}")
            for root, _, files in os.walk(udir):
                for f in sorted(files):
                    if f.endswith(".md"):
                        path = os.path.join(root, f)
                        with open(path, "r", encoding="utf-8") as fp:
                            content = fp.read()
                        matches = pattern.findall(content)
                        if matches:
                            found_violations.append((path, matches))

        self.assertEqual(
            len(found_violations),
            0,
            f"Found invalid \\mathrm{{..._...}} in modular units: {found_violations}",
        )


if __name__ == "__main__":
    unittest.main()




