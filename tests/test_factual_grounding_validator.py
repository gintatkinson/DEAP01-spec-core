"""
Unit tests for Check 23 (Factual Grounding & Parametric SSOT Gate) and FactualGroundingValidator.

Verifies:
1. Registration in AGGREGATING_VALIDATORS and scripts/verify_downstream_baseline.py Check 23 integration.
2. Upstream clean landing zone (empty schema/) passes gracefully with 0 findings.
3. Rejection of ungrounded control surface count drift ('factual-grounding-numeric-drift').
4. Rejection of ungrounded structural descriptors ('factual-grounding-numeric-drift').
5. Rejection of fabricated catapult launch acceleration / numeric drift ('factual-grounding-numeric-drift').
6. Rejection of unverified protocols ('factual-grounding-unverified-protocol').
7. Acceptance of grounded parameters, declared protocols, and explicit SSOT citations.
8. Rejection of autonomous arming sequence diagrams without prior HITL C2 command ('factual-grounding-temporal-safety-violation').
9. Acceptance of sequence diagrams with proper temporal HITL operator authorization before physical arming.
10. Contextual filtering: skipping non-normative sections (Glossary, MCDA trade studies) and comments.
"""

import os
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.factual_grounding_validator import FactualGroundingValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import check_factual_grounding

SAMPLE_GROUND_TRUTH_SYSML = """package AutonomousVehicle_SSOT {
    doc /* SSOT for Autonomous Flight Vehicle */

    attribute ruddervatorCount : Integer = 4;
    attribute tailConfiguration : String = "X-tail";
    attribute catapultLaunchLimitG : Real = 12.0;
    attribute maxGLoad : Real = 12.0;

    part def Airframe {
        attribute massKg : Real = 25.0;
    }

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
    }
}
"""

SAMPLE_BOM_MARKDOWN = """# Level 0 OEM Ground-Truth Bill of Materials

## Actuation & Empennage
| Component | Quantity | Description |
| :--- | :--- | :--- |
| Ruddervator Actuators | 4 | Independent high-bandwidth control surface actuators |
| Control Surfaces | 4 | X-tail ruddervator arrangement |
| Catapult Launch Limit | 12g | Maximum allowable rail launch acceleration |
| Primary Serial Bus | RS-485 | Differential half-duplex avionics bus |
"""


class TestCheck23FactualGroundingValidator(unittest.TestCase):
    def setUp(self):
        self.validator = FactualGroundingValidator()

    def test_registered_in_aggregating_validators(self):
        """Verify FactualGroundingValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(FactualGroundingValidator, AGGREGATING_VALIDATORS)

    def test_upstream_clean_landing_zone(self):
        """Verify clean landing zone (empty schema/ or only .gitkeep) returns 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            with open(os.path.join(docs_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write("# Clean landing zone test\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_rejects_control_surface_count_drift(self):
        """Verify rejection when document claims 2 ruddervators while schema specifies 4."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Flight Control Surface Allocation

## Control Allocation Architecture
The vehicle employs 2 independent ruddervators to achieve pitch and yaw stabilization during flight.
"""
            with open(os.path.join(docs_dir, "FEAT_01_FLIGHT_CONTROL.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("2 independent ruddervators" in str(f) for f in findings))

    def test_rejects_structural_descriptor_drift(self):
        """Verify rejection when document asserts V-tail when schema specifies X-tail (4 surfaces)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Concept of Operations

## Airframe Empennage Geometry
The airframe is configured with a conventional V-tail empennage for aerodynamic pitch/yaw control.
"""
            with open(os.path.join(docs_dir, "CONOPS_EMPENNAGE.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("V-tail" in str(f) for f in findings))

    def test_rejects_fabricated_numeric_quantities(self):
        """Verify rejection of fabricated 15-20g / 18g catapult launch load when schema specifies 12g limit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema", "extracted")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "oem_spec.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_BOM_MARKDOWN)

            doc_md = """# Launch Operations Specification

## Rail Launch Phase
The pneumatic catapult system accelerates the vehicle under an 15-20g launch load profile.
"""
            with open(os.path.join(docs_dir, "CONOPS_LAUNCH.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("15-20g" in str(f) for f in findings))

    def test_rejects_unverified_protocols(self):
        """Verify rejection of ungrounded protocol claims (e.g. STANAG 4586 or MIL-STD-1553 not in schema)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "icds")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Interface Control Document

## External Datalink Protocols
The GCS datalink operates over STANAG 4586 compliant message structures and MIL-STD-1553 dual-redundant bus interfaces.
"""
            with open(os.path.join(docs_dir, "ICD_EXTERNAL_DATALINK.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-unverified-protocol", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertIn("STANAG 4586", findings_text)
            self.assertIn("MIL-STD-1553", findings_text)

    def test_accepts_grounded_specifications_with_ssot_citations(self):
        """Verify that grounded protocols, 4 ruddervators, 12g limit, and explicit SSOT citations pass cleanly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """---
source_references:
  - schema/model.sysml
---
# Flight Control & Avionics

## Architecture Baseline
<!-- Source: schema/model.sysml -->
The flight controller interfaces with 4 ruddervators arranged in an X-tail configuration over RS-485.
Telemetry is streamed using MAVLink protocols.
The rail launch acceleration limit is strictly enforced at 12g.
"""
            with open(os.path.join(docs_dir, "FEAT_01_GROUNDED.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_rejects_autonomous_arming_sequence_diagram(self):
        """Verify that a sequence diagram sending physical arming signal without prior HITL C2 consent is rejected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "use-cases")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Payload Deployment Use Case

## Autonomous Ignition Sequence
```mermaid
sequenceDiagram
    autonumber
    participant Autopilot as FCS
    participant SafeArm as FiringCircuit
    participant Motor as SolidRocketMotor

    Autopilot ->> SafeArm: Arm_Circuit
    SafeArm ->> Motor: Fire_Pulse
```
"""
            with open(os.path.join(docs_dir, "UC_01_ARMING.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-temporal-safety-violation", rule_ids)
            self.assertTrue(any("Autonomous arming sequence detected" in str(f) for f in findings))

    def test_accepts_hitl_authorized_sequence_diagram(self):
        """Verify that a sequence diagram with explicit Operator C2 arming consent preceding physical signal passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "use-cases")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Payload Deployment Use Case

## Human Authorized Ignition Sequence
```mermaid
sequenceDiagram
    autonumber
    actor Pilot as GCS_Operator
    participant Autopilot as FCS
    participant SafeArm as FiringCircuit
    participant Motor as SolidRocketMotor

    Pilot ->> Autopilot: Arm_Command (Consent_Granted)
    Autopilot ->> SafeArm: Arm_Circuit
    Autopilot ->> SafeArm: Fire_Pulse
    SafeArm ->> Motor: Ignite_Motor
```
"""
            with open(os.path.join(docs_dir, "UC_02_HITL_ARMING.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_skips_non_normative_mcda_trade_study_and_glossary(self):
        """Verify that trade study alternatives and glossary entries do not trigger false positive findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Concept of Operations

## MCDA Trade Study: Empennage Options
| Candidate Option | Architecture | Status |
| :--- | :--- | :--- |
| Option A | 2 independent ruddervators (V-tail) | REJECTED: Insufficient crosswind authority |
| Option B | 4 ruddervators (X-tail) | SELECTED: Baseline architecture |

## MCDA Trade Study: Protocol Alternatives
Evaluating candidate protocols: STANAG 4586 and MIL-STD-1553 were analyzed but rejected due to SWaP constraints.

## Glossary & Acronyms
- STANAG 4586: Standard Interfaces of UAV Control System
- MIL-STD-1553: Military Standard Serial Bus
"""
            with open(os.path.join(docs_dir, "CONOPS_TRADE_STUDIES.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

    def test_excludes_retrospective_defect_reports_and_audit_summaries(self):
        """Verify that retrospective defect reports under docs/reports/defects/ and *AUDIT.md / *audit*.md files are excluded."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            defects_dir = os.path.join(tmpdir, "docs", "reports", "defects")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(defects_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            # Retrospective defect report quoting historical defects
            with open(os.path.join(defects_dir, "DEFECT_001_VTAIL.md"), "w", encoding="utf-8") as f:
                f.write("# Defect Report 001\nQuoting ungrounded claim: 2 ruddervators V-tail and STANAG 4586\n")

            # Audit summary file ending in AUDIT.md
            with open(os.path.join(reports_dir, "PARITY_AUDIT.md"), "w", encoding="utf-8") as f:
                f.write("# Parity Audit Summary\nHistorical violation noted: 15-20g launch load and MIL-STD-1553\n")

            # Audit summary file matching *audit*.md
            with open(os.path.join(conops_dir, "conops_audit_summary.md"), "w", encoding="utf-8") as f:
                f.write("# ConOps Audit Summary\nAudit finding: autonomous arming without HITL consent\n```mermaid\nsequenceDiagram\nAutopilot ->> FiringCircuit: Arm_Circuit\n```\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_closed_world_rejects_arbitrary_ungrounded_descriptor(self):
        """Verify that an arbitrary ungrounded compound structural descriptor (e.g. abc-tail and theta-wing) fails Check 23."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            # Schema declares tailConfiguration attribute and physical Wing part def
            schema_sysml = """package MinimalVehicle_SSOT {
    attribute maxGLoad : Real = 12.0;
    attribute tailConfiguration : String = "X-tail";
    part def Wing {
        attribute spanM : Real = 3.5;
    }
    part def Airframe {
        attribute massKg : Real = 25.0;
    }
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            doc_md = """# Concept of Operations

## Airframe Empennage Geometry
The airframe is configured with an ungrounded abc-tail empennage and theta-wing geometry for aerodynamic control.
The system operates in real-time mode with a two-step valid-range check, single-stage ignition, three-state logic, and multi-mode sign-off.
"""
            with open(os.path.join(docs_dir, "CONOPS_EMPENNAGE.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("abc-tail" in str(f) for f in findings))
            self.assertTrue(any("theta-wing" in str(f) for f in findings))
            findings_text = " ".join(str(f) for f in findings)
            self.assertNotIn("real-time", findings_text)
            self.assertNotIn("two-step", findings_text)
            self.assertNotIn("valid-range", findings_text)
            self.assertNotIn("multi-mode", findings_text)
            self.assertNotIn("sign-off", findings_text)
            self.assertNotIn("single-stage", findings_text)
            self.assertNotIn("three-state", findings_text)

    def test_tracer_and_signal_identifiers_excluded_from_structural_descriptors(self):
        """Verify that signal/interface identifiers, requirement codes, and architectural tracer tags
        (e.g. SIG-ESAD, SIG-ONBOARDCOMPUTER, REQ-01-AIRFRAME, CONN-BATTERY, FEAT-01-TAIL, US-01-WING)
        are NOT classified as ungrounded compound structural descriptors, while actual ungrounded
        structural descriptors (like abc-tail) continue to be flagged.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "specs")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package AutonomousVehicle_SSOT {
    attribute maxGLoad : Real = 12.0;
    attribute tailConfiguration : String = "X-tail";

    part def ESAD {
        attribute massKg : Real = 1.2;
    }

    part def OnboardComputer {
        attribute powerW : Real = 45.0;
    }

    part def Airframe {
        attribute massKg : Real = 25.0;
    }

    part def Battery {
        attribute voltageV : Real = 28.0;
    }

    part def Wing {
        attribute spanM : Real = 3.5;
    }
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            doc_md = """# Interface Control & Traceability Specification

## Signals and Interfaces
| Signal ID | Type | Description |
| :--- | :--- | :--- |
| SIG-ESAD | Discrete | Arming status and fire inhibit |
| SIG-ONBOARDCOMPUTER | CAN | Telemetry health bus |

## Architectural Tracing
The airframe satisfies the following requirements and traces:
- Requirement: REQ-01-AIRFRAME
- Connector: CONN-BATTERY
- Feature: FEAT-01-TAIL
- User Story: US-01-WING
- Use Case: UC-01
- Epic: EPIC-01
- Operator Scenario: OP-01
- Safety Scenario: SC-01
- Rule: RULE-01
- Verification Test: TEST-01
- Operational Safety Objective: OSO-01
- Unsafe Control Action: UCA-01

## Empennage Configuration
The vehicle is equipped with an ungrounded abc-tail empennage.
"""
            with open(os.path.join(docs_dir, "ICD_SPEC.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            findings_text = " ".join(str(f) for f in findings)
            # Verify tracer tags and signal identifiers are NOT falsely flagged as structural descriptors
            self.assertNotIn("SIG-ESAD", findings_text)
            self.assertNotIn("SIG-ONBOARDCOMPUTER", findings_text)
            self.assertNotIn("REQ-01-AIRFRAME", findings_text)
            self.assertNotIn("01-AIRFRAME", findings_text)
            self.assertNotIn("CONN-BATTERY", findings_text)
            self.assertNotIn("FEAT-01-TAIL", findings_text)
            self.assertNotIn("01-TAIL", findings_text)
            self.assertNotIn("US-01-WING", findings_text)
            self.assertNotIn("01-WING", findings_text)
            self.assertNotIn("UC-01", findings_text)
            self.assertNotIn("EPIC-01", findings_text)
            self.assertNotIn("OP-01", findings_text)
            self.assertNotIn("SC-01", findings_text)
            self.assertNotIn("RULE-01", findings_text)
            self.assertNotIn("TEST-01", findings_text)
            self.assertNotIn("OSO-01", findings_text)
            self.assertNotIn("UCA-01", findings_text)

            # Verify actual ungrounded structural descriptor IS caught
            self.assertTrue(any("abc-tail" in str(f) for f in findings))


if __name__ == "__main__":
    unittest.main()


