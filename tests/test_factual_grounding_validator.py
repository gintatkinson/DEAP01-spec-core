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

            # Schema declares tailConfiguration and wingConfiguration config targets
            schema_sysml = """package MinimalVehicle_SSOT {
    attribute maxGLoad : Real = 12.0;
    attribute tailConfiguration : String = "X-tail";
    attribute wingConfiguration : String = "delta-wing";
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

    def test_count_targets_excludes_registers_indices_and_thresholds(self):
        """Verify that integer attributes representing registers, indices, status codes, masks,
        baud rates, or thresholds are excluded from count_targets, while genuine component counts
        (e.g. ending in count, qty, surfaces, channels, actuators) are preserved.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute ruddervatorCount : Integer = 4;
    attribute controlSurfaces : Integer = 4;
    attribute rfChannels : Integer = 2;
    attribute faultRegister : Integer = 16;
    attribute statusRegister : Integer = 1;
    attribute statusCode : Integer = 200;
    attribute channelIndex : Integer = 1;
    attribute channelOffset : Integer = 8;
    attribute actuatorMask : Integer = 15;
    attribute surfaceThreshold : Integer = 10;
    attribute telemetryBaud : Integer = 115200;
    attribute bufferChars : Integer = 256;
    attribute payloadBits : Integer = 32;
    attribute actuatorId : Integer = 3;
    attribute systemState : Integer = 2;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # Document mentions various non-count items with differing counts, but only ruddervators drifts
            doc_md = """# Flight Control Configuration
The subsystem uses 2 fault registers, 4 channel indices, 1 actuator mask, 8 buffer chars, and 2 ruddervators.
"""
            with open(os.path.join(docs_dir, "FEAT_CONFIG.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            findings_text = " ".join(str(f) for f in findings)
            # Must NOT flag registers, indices, masks, or chars as component count contradictions
            self.assertNotIn("fault register", findings_text.lower())
            self.assertNotIn("channel indic", findings_text.lower())
            self.assertNotIn("actuator mask", findings_text.lower())
            self.assertNotIn("buffer char", findings_text.lower())

            # MUST flag the actual count drift for 2 ruddervators vs 4
            self.assertTrue(any("2 ruddervators" in str(f) for f in findings))

    def test_config_targets_excludes_metadata_strings(self):
        """Verify that string attributes representing metadata (name, title, description, doc,
        note, ref, poly, init, vector, standard, baseline, variants, camera) are excluded from
        config_targets, while genuine configuration attributes (configuration, config, layout,
        arrangement, topology, architecture, type) are retained.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute tailConfiguration : String = "X-tail";
    attribute avionicsArchitecture : String = "distributed";
    attribute vehicleName : String = "Eagle-One";
    attribute systemTitle : String = "Autonomous System";
    attribute configDescription : String = "Primary vehicle spec";
    attribute systemDoc : String = "Reference doc";
    attribute safetyNote : String = "Critical safety note";
    attribute specRef : String = "DO-178C";
    attribute crcPoly : String = "0x1021";
    attribute stateInit : String = "STANDBY";
    attribute testVector : String = "VEC-001";
    attribute standardRef : String = "MIL-STD";
    attribute architectureBaseline : String = "REV-B";
    attribute payloadVariants : String = "EO-IR";
    attribute payloadCamera : String = "Sony";
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # Document mentions different metadata strings and a contradictory tail descriptor
            doc_md = """# Concept of Operations
The vehicle named Falcon-Two conforms to title UAV-Platform with doc UserGuide.
It features an ungrounded V-tail configuration.
"""
            with open(os.path.join(docs_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            findings_text = " ".join(str(f) for f in findings)
            # Metadata strings must not be flagged as contradictory configuration descriptors
            self.assertNotIn("Falcon-Two", findings_text)
            self.assertNotIn("UAV-Platform", findings_text)
            self.assertNotIn("UserGuide", findings_text)

            # Genuine configuration target (tailConfiguration: X-tail) must flag V-tail
            self.assertTrue(any("V-tail" in str(f) for f in findings))

    def test_protocol_and_standard_number_exclusions(self):
        """Verify that numbers preceded or followed by standard names (e.g. RS-485, RS485,
        EIA-485, 485 bus, 485 Commands, MIL-STD-461, STANAG 4187, 4187) are not matched as
        physical component counts.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "icds")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute busCount : Integer = 2;
    attribute commandCount : Integer = 10;
    attribute channelCount : Integer = 2;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            doc_md = """# Interface Control Document
## Protocol and Bus Interfaces
- Differential serial communication utilizes RS-485 bus interfaces.
- The telemetry line connects via RS485 bus transceivers.
- Legacy hardware supports EIA-485 bus lines.
- Payload commands are transferred using 485 bus links.
- The flight control computer processes 485 Commands over the avionics bus.
- Electromagnetic susceptibility meets MIL-STD-461 requirements.
- Interoperability conforms to STANAG 4187 specifications.
- Firing logic meets 4187 safety requirements.
"""
            with open(os.path.join(docs_dir, "ICD_PROTOCOLS.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            # None of RS-485, RS485, EIA-485, 485 bus, 485 Commands, MIL-STD-461, STANAG 4187, 4187
            # should be flagged as numeric drift contradicting 2 buses, 10 commands, or 2 channels.
            numeric_drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertEqual(numeric_drift_findings, [])

    def test_protocol_exclusion_does_not_mask_genuine_count_drift(self):
        """Verify that genuine count drift on a line containing protocol numbers is still detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "icds")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute busCount : Integer = 2;
    port c2 : RS-485;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # Line contains both an RS-485 mention AND an incorrect count of 4 buses
            doc_md = """# Interface Control Document
The architecture implements 4 redundant buses over an RS-485 bus network.
"""
            with open(os.path.join(docs_dir, "ICD_BUS.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            # Must detect 4 redundant buses contradicting 2 buses
            numeric_drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertEqual(len(numeric_drift_findings), 1)
            self.assertTrue(any("4 redundant buses" in str(f) for f in numeric_drift_findings))
            # Must not falsely claim 485 buses contradicts 2
            self.assertFalse(any("485" in str(f) for f in numeric_drift_findings))

    def test_markdown_bom_count_and_config_target_exclusions(self):
        """Verify that Markdown BOM table rows and bullets exclude registers/metadata and retain
        genuine count_targets (count, qty, surfaces, channels, actuators) and config_targets.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema", "extracted")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            bom_md = """# Bill of Materials
| Property | Value | Description |
| :--- | :--- | :--- |
| Ruddervator Actuators | 4 | Four independent actuators |
| Control Surfaces | 4 | Empennage surfaces |
| Empennage Layout | X-tail | Empennage geometry |
| Fault Register | 16 | Bitmask status register |
| Status Code | 200 | HTTP OK status |
| Baud Rate | 115200 | Serial bitrate |
| Channel Index | 1 | Zero-based channel index |
| System Title | SkyWatcher | Marketing name |
| Config Description | Baseline spec | Specification summary |

## System Parameters
- Actuators Quantity: 4
- Surface Threshold: 10
- Component Name: Airframe-Alpha
"""
            with open(os.path.join(schema_dir, "bom.md"), "w", encoding="utf-8") as f:
                f.write(bom_md)

            doc_md = """# Feature Specification
The system contains 2 fault registers, 4 channel indices, and title Drone-Beta.
The airframe is configured with 2 ruddervators in a V-tail layout.
"""
            with open(os.path.join(docs_dir, "FEAT_SPEC.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            findings_text = " ".join(str(f) for f in findings)
            # Must NOT flag registers, indices, or titles
            self.assertNotIn("fault register", findings_text.lower())
            self.assertNotIn("channel indic", findings_text.lower())
            self.assertNotIn("Drone-Beta", findings_text)

            # MUST flag 2 ruddervators drift vs 4
            self.assertTrue(any("2 ruddervators" in str(f) for f in findings))
            # MUST flag V-tail drift vs X-tail
            self.assertTrue(any("V-tail" in str(f) for f in findings))

    def test_lower_bound_distinguished_from_upper_bound_allows_valid_range(self):
        """Verify that lower bounds (e.g. hvRangeMinV: 0.0V) are distinguished from upper bounds:
        a claim of 50 V exceeds 0.0V, but is completely valid for a minimum bound and must NOT trigger
        'Fabricated numeric quantity 50 V exceeds schema ground truth limit (0.0v)'.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute hvRangeMinV : Real = 0.0;
    attribute hvRangeMaxV : Real = 100.0;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # 50 V is between min 0.0V and max 100.0V -> completely valid!
            doc_md = """# Electrical Power Architecture
The high-voltage subsystem operates in the hv range at 50 V.
"""
            with open(os.path.join(docs_dir, "FEAT_POWER.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            # 50 V must NOT be flagged as exceeding 0.0V
            numeric_drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertEqual(numeric_drift_findings, [])

    def test_lower_bound_strictly_rejects_values_below_minimum(self):
        """Verify that values strictly less than a declared lower bound (val < min) are rejected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute minOperationalVoltageV : Real = 18.0;
    attribute maxOperationalVoltageV : Real = 36.0;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # Claim of 12 V is strictly below the lower bound of 18.0 V -> violation!
            doc_md = """# Power Regulation
The avionics bus operates at 12 V during emergency low-power state.
"""
            with open(os.path.join(docs_dir, "FEAT_REG.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            numeric_drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertEqual(len(numeric_drift_findings), 1)
            finding_str = str(numeric_drift_findings[0])
            self.assertIn("12 V", finding_str)
            self.assertIn("lower bound", finding_str)

    def test_part_def_tokens_not_scoped_to_structural_nouns_preventing_prose_false_positives(self):
        """Verify that arbitrary part def tokens (power, segment, fiber, sensor, launch) are NOT added
        to structural_nouns, preventing normal technical prose (carbon-fiber, post-launch, full-power,
        multi-segment, optical-sensor) from being falsely flagged, while ungrounded descriptors on
        declared config_targets (e.g. abc-tail from tailConfiguration) continue to be rejected.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            schema_sysml = """package Vehicle_SSOT {
    attribute tailConfiguration : String = "X-tail";

    part def PowerSupply {}
    part def FiberOptics {}
    part def LaunchRail {}
    part def MultiSegmentWing {}
    part def OpticalSensor {}
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(schema_sysml)

            # Normal prose containing carbon-fiber, post-launch, full-power, multi-segment, optical-sensor
            # along with an ungrounded abc-tail descriptor
            doc_md = """# Vehicle Specification
The structure uses carbon-fiber composites and operates at full-power during post-launch flight.
The flight path uses a multi-segment profile with optical-sensor guidance.
The vehicle features an ungrounded abc-tail configuration.
"""
            with open(os.path.join(docs_dir, "FEAT_MATERIALS.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            findings_text = " ".join(str(f) for f in findings)
            # Must NOT flag prose tokens
            self.assertNotIn("carbon-fiber", findings_text)
            self.assertNotIn("full-power", findings_text)
            self.assertNotIn("post-launch", findings_text)
            self.assertNotIn("multi-segment", findings_text)
            self.assertNotIn("optical-sensor", findings_text)

            # MUST flag ungrounded structural descriptor on config target
            self.assertTrue(any("abc-tail" in str(f) for f in findings))


if __name__ == "__main__":
    unittest.main()



