"""
Unit tests for Check 23 (Factual Grounding & Numeric Provenance Gate) and FactualGroundingValidator.

Verifies:
1. Registration in AGGREGATING_VALIDATORS and scripts/verify_downstream_baseline.py Check 23 integration.
2. test_check23_rejects_ungrounded_vtail_with_4_ruddervators: Rejection of ungrounded V-tail claims when BOM specifies 4 ruddervators ('factual-grounding-numeric-drift').
3. test_check23_rejects_fabricated_gload_claims: Rejection of fabricated 15-20g G-load claims without OEM source ('factual-grounding-numeric-drift').
4. test_check23_rejects_unverified_stanag_protocol: Rejection of ungrounded/undeclared STANAG protocol claims ('factual-grounding-unverified-protocol').
5. test_check23_rejects_autonomous_arming_sequence: Rejection of autonomous arming sequences without prior human operator C2 consent ('factual-grounding-temporal-safety-violation').
6. test_check23_accepts_fully_grounded_specifications_with_citations: Acceptance of grounded parameters, declared protocols, and explicit SSOT citations.
7. test_check23_upstream_clean_landing_zone: Graceful zero findings on clean landing zones (empty schema/ or .gitkeep).
8. test_check23_baseline_verification_integration: Integration verification for check_factual_grounding and check_factual_grounding_and_provenance in verify_downstream_baseline.py.
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
from scripts.verify_downstream_baseline import (
    check_factual_grounding,
    check_factual_grounding_and_provenance,
)

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


class TestCheck23FactualGroundingGate(unittest.TestCase):
    def setUp(self):
        self.validator = FactualGroundingValidator()

    def test_registered_in_aggregating_validators(self):
        """Verify FactualGroundingValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(FactualGroundingValidator, AGGREGATING_VALIDATORS)

    def test_check23_rejects_ungrounded_vtail_with_4_ruddervators(self):
        """Test rejection of ungrounded V-tail claims when BOM/SysML specifies 4 ruddervators (X-tail)."""
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

    def test_check23_rejects_fabricated_gload_claims(self):
        """Test rejection of fabricated 15-20g G-load claims without OEM source."""
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

    def test_check23_rejects_unverified_stanag_protocol(self):
        """Test rejection of undeclared STANAG protocol claims not in schema."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "icds")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            doc_md = """# Interface Control Document

## External Datalink Protocols
The GCS datalink operates over STANAG 4586 compliant message structures.
"""
            with open(os.path.join(docs_dir, "ICD_EXTERNAL_DATALINK.md"), "w", encoding="utf-8") as f:
                f.write(doc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-unverified-protocol", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertIn("STANAG 4586", findings_text)

    def test_check23_rejects_autonomous_arming_sequence(self):
        """Test rejection of autonomous arming sequence without human C2 command in Mermaid sequence diagram."""
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

    def test_check23_accepts_fully_grounded_specifications_with_citations(self):
        """Test acceptance of fully grounded specifications with valid SSOT citations and HITL sequence diagram."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            use_cases_dir = os.path.join(tmpdir, "docs", "use-cases")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)
            os.makedirs(use_cases_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            feat_md = """---
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
                f.write(feat_md)

            uc_md = """# Payload Deployment Use Case

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
            with open(os.path.join(use_cases_dir, "UC_02_HITL_ARMING.md"), "w", encoding="utf-8") as f:
                f.write(uc_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_check23_upstream_clean_landing_zone(self):
        """Test acceptance of clean upstream landing zone (empty schema or .gitkeep)."""
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

    def test_check23_baseline_verification_integration(self):
        """Test check_factual_grounding_and_provenance and check_factual_grounding functions in clean and failing workspaces."""
        # 1. Clean workspace
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                pass

            # Both alias and primary function should return cleanly without exit
            check_factual_grounding(tmpdir)
            check_factual_grounding_and_provenance(tmpdir)

        # 2. Failing workspace with ungrounded assertion
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            with open(os.path.join(docs_dir, "CONOPS_EMPENNAGE.md"), "w", encoding="utf-8") as f:
                f.write("# Empennage\nThe airframe is configured with a conventional V-tail empennage.\n")

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

            with self.assertRaises(SystemExit) as cm_alias:
                check_factual_grounding_and_provenance(tmpdir)
            self.assertEqual(cm_alias.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
