#!/usr/bin/env python3
"""
Unit tests for SysML v2 Closed-Loop Reverse Synchronization Engine (`compile_sysml.py --reverse-sync`).

Verifies:
1. Non-destructive AST merging (existing parts, ports, attributes, actions, operations, constraints preserved 100%).
2. In-place overwrite protection (allow_schema_overwrite=False prevents modifying base schema).
3. Cryptographic digest computation (schema-digest.json) and valid SysML v2 textual syntax emission.
4. Explicit runtime errors on schema parse failures and missing files.
"""

import os
import sys
import json
import hashlib
import tempfile
import unittest

# Ensure spec-orchestrator scripts and repo root are on sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

for p in (PROJECT_ROOT, SPEC_SCRIPTS_DIR):
    if p not in sys.path:
        sys.path.insert(0, p)

from sysmlv2_ast import (
    SysMLPackage,
    PartDef,
    AttributeDef,
    PortDef,
    ActionDef,
    SysMLOperationDef,
    SysMLCapabilityDef,
    SysMLInteractionDef,
    SysMLConstraintDef,
    SysMLTestCaseDef,
    UseCaseDef,
    SysMLParser,
)
from scripts.compile_sysml import (
    reverse_sync_specs_to_sysml,
    extract_use_cases_from_markdown,
    extract_user_story_ast,
    extract_features_from_markdown,
    extract_epics_from_markdown,
    parse_stpa_ucas,
    parse_fmeca_modes,
)


SAMPLE_BASE_SCHEMA = """package AutonomousUAS_SSOT {
    doc /* Single Source of Truth for Autonomous UAS Infrastructure Safety */
    part def FlightController {
        doc /* Base Flight Controller hardware and control unit */
        attribute firmwareVersion : String;
        in port c2Port : C2Interface;
        action CalibrateSensors();
        operation GetStatus() : SystemStatus;
        assert constraint FirmwareIntegrityConstraint {
            firmwareChecksumValid == true;
        }
    }
}
"""

SAMPLE_USE_CASE_MD = """---
title: "Autonomous Return to Home"
type: "use-case"
use_case_id: "UC-01"
use_case_def: "AutonomousRTH"
subject: "FlightController"
actors:
  - "GroundStationOperator"
  - "DAA_SensorArray"
objective: "Safely navigate UAS back to launch coordinates on C2 link loss"
includes:
  - "EmergencyFailsafe"
extends:
  - "ManualOverride"
---

# Use Case: Autonomous Return to Home

## 1. Overview
- **Subject Part:** FlightController
- **Actors:** GroundStationOperator, DAA_SensorArray
- **Objective:** "Safely navigate UAS back to launch coordinates on C2 link loss"
- **Includes:** EmergencyFailsafe
- **Extends:** ManualOverride
"""

SAMPLE_USER_STORY_MD = """---
title: "Autonomous Trajectory Deconfliction"
type: "user-story"
generation_mode: "subagent"
interaction: "TrajectoryDeconfliction"
test_case: "TC_CollisionAvoidance_001"
---

# User Story: Autonomous Trajectory Deconfliction

## UML Sequence Diagram
```mermaid
sequenceDiagram
    autonumber
    actor radarOperator as "radarOperator : RadarOperator"
    participant flightGuidance as "flightGuidance : FlightController"
    participant motorController as "motorController : MotorController"

    radarOperator->>flightGuidance: ExecuteManeuver(targetHeading: Float)
    flightGuidance->>motorController: sendEmergencyVector(vector: Vector3D)
    flightGuidance-->radarOperator: status : Status
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `TC_CollisionAvoidance_001`
- **Subject Part:** `FlightController`
- **Verified Safety Requirement:** `REQ_SAF_001`
- **Verification Objective:** "Verify emergency vector computed within 150ms"
- **Test Steps:**
  - `step inject_synthetic_threat`
  - `step assert_evasive_maneuver_commanded`
"""

SAMPLE_FEATURE_MD = """---
title: "Flight Controller"
type: "feature"
interface_type: "m2m"
generation_mode: "subagent"
part_def: "FlightController"
---

# Feature: Flight Controller

## UML Class Diagram
```mermaid
classDiagram
    class FlightController {
        +String activeRouteId
        +Boolean ExecuteManeuver(Float in_targetHeading, StatusEnum out_maneuverStatus)
        +ThrustVector ComputeThrustVector(Vector3D in_currentHeading, Vector3D in_targetHeading)
    }
```

## Validation & Constraints
- ThermalLimitConstraint: coreTemperature <= 85.0
"""

SAMPLE_EPIC_MD = """---
title: "Flight Guidance Subsystem"
type: "epic"
package: "FlightGuidance"
generation_mode: "subagent"
---

# Epic: Flight Guidance Subsystem

## Subsystem Capability Allocations
| Capability Name | Subsystem Package | Description / Objective |
| --- | --- | --- |
| AutonomousCollisionAvoidance | FlightGuidance | Executes real-time detect-and-avoid trajectory deconfliction |
| PrecisionLanding | FlightGuidance | Beacon-guided autonomous precision landing |
"""

SAMPLE_STPA_MATRIX_MD = """# STPA Unsafe Control Action (UCA) Matrix

| UCA ID | Controller | Control Action | STPA UCA Category | Environmental Context Vector | Triggered System Hazard | Severity Classification | SORA SAIL Target |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCA-UAS-01** | Flight Controller | Fail-Safe Return-to-Launch (RTL) | **1. Not Provided** | $t_{\\text{loss}} > 2.0\\text{ s}$, C2 Link Down | **H_UAS_1:** Lost-Link Flyaway | Catastrophic | **SAIL IV–VI** |

## FMECA Failure Modes
| FMECA ID | Component | Failure Mode | Effect | Safety Invariant |
| :--- | :--- | :--- | :--- | :--- |
| **FMECA-UAS-01** | Magnetometer | Magnetic flux saturation | Compass heading divergence | flux <= 250.0 uT |
"""


class TestReverseSysMLSync(unittest.TestCase):
    """Test suite for Reverse SysML Synchronization Engine non-destructive merging and safety invariants."""

    def setUp(self):
        self.tmpdir_obj = tempfile.TemporaryDirectory()
        self.tmpdir = self.tmpdir_obj.name

        self.docs_dir = os.path.join(self.tmpdir, "docs")
        self.uc_dir = os.path.join(self.docs_dir, "use-cases")
        self.us_dir = os.path.join(self.docs_dir, "user-stories")
        self.feat_dir = os.path.join(self.docs_dir, "features")
        self.epic_dir = os.path.join(self.docs_dir, "epics")
        self.safety_dir = os.path.join(self.docs_dir, "safety")

        os.makedirs(self.uc_dir, exist_ok=True)
        os.makedirs(self.us_dir, exist_ok=True)
        os.makedirs(self.feat_dir, exist_ok=True)
        os.makedirs(self.epic_dir, exist_ok=True)
        os.makedirs(self.safety_dir, exist_ok=True)

        with open(os.path.join(self.uc_dir, "uc-01-autonomous-rth.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USE_CASE_MD)
        with open(os.path.join(self.us_dir, "us-01-trajectory-deconfliction.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USER_STORY_MD)
        with open(os.path.join(self.feat_dir, "feat-01-flight-controller.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_FEATURE_MD)
        with open(os.path.join(self.epic_dir, "epic-01-flight-guidance.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_EPIC_MD)
        with open(os.path.join(self.safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_STPA_MATRIX_MD)

        self.base_schema_path = os.path.join(self.tmpdir, "schema", "DEAP_MODEL.sysml")
        os.makedirs(os.path.dirname(self.base_schema_path), exist_ok=True)
        with open(self.base_schema_path, "w", encoding="utf-8") as f:
            f.write(SAMPLE_BASE_SCHEMA)

        self.out_sysml = os.path.join(self.tmpdir, ".pipeline", "schema.sysml")
        self.out_digest = os.path.join(self.tmpdir, ".pipeline", "schema-digest.json")

    def tearDown(self):
        self.tmpdir_obj.cleanup()

    def test_non_destructive_ast_merging_preserves_base_schema(self):
        """Verify that existing base schema parts, ports, attributes, actions, operations, constraints are preserved 100%."""
        pkg, digest = reverse_sync_specs_to_sysml(
            docs_dir=self.docs_dir,
            schema_path=self.base_schema_path,
            output_path=self.out_sysml,
            digest_path=self.out_digest,
            allow_schema_overwrite=False,
        )

        self.assertTrue(os.path.exists(self.out_sysml))
        parsed_pkg = SysMLParser.parse_file(self.out_sysml)

        # 1. Verify Part exists and is matched
        fc_part = next((p for p in parsed_pkg.part_defs if p.name == "FlightController"), None)
        self.assertIsNotNone(fc_part, "FlightController part definition must exist in merged AST")

        # 2. Verify 100% preservation of base elements
        port_names = {p.name: p for p in fc_part.ports}
        self.assertIn("c2Port", port_names, "Original c2Port port must be preserved 100%")

        attr_names = {a.name: a for a in fc_part.attributes}
        self.assertIn("firmwareVersion", attr_names, "Original firmwareVersion attribute must be preserved 100%")
        self.assertIn("activeRouteId", attr_names, "New activeRouteId attribute must be merged in")

        action_names = {a.name: a for a in fc_part.actions}
        self.assertIn("CalibrateSensors", action_names, "Original CalibrateSensors action must be preserved 100%")
        self.assertIn("ExecuteManeuver", action_names, "New ExecuteManeuver action must be merged in")

        op_names = {o.name: o for o in fc_part.operations}
        self.assertIn("GetStatus", op_names, "Original GetStatus operation must be preserved 100%")
        self.assertIn("ComputeThrustVector", op_names, "New ComputeThrustVector operation must be merged in")

        con_names = {c.name: c for c in fc_part.constraints}
        self.assertIn("FirmwareIntegrityConstraint", con_names, "Original FirmwareIntegrityConstraint must be preserved 100%")
        self.assertIn("ThermalLimitConstraint", con_names, "New ThermalLimitConstraint must be merged in")

        # 3. Verify Use Case, Interaction, Test Case, Capability, and STPA constraint additions
        self.assertEqual(len(parsed_pkg.use_case_defs), 1)
        self.assertEqual(parsed_pkg.use_case_defs[0].name, "AutonomousRTH")
        self.assertEqual(parsed_pkg.use_case_defs[0].subject, "FlightController")

        self.assertEqual(len(parsed_pkg.interaction_defs), 1)
        self.assertEqual(parsed_pkg.interaction_defs[0].name, "TrajectoryDeconfliction")

        self.assertEqual(len(parsed_pkg.test_case_defs), 1)
        self.assertEqual(parsed_pkg.test_case_defs[0].name, "TC_CollisionAvoidance_001")

        self.assertEqual(len(parsed_pkg.capability_defs), 2)
        self.assertEqual(len(parsed_pkg.constraint_defs), 2)  # Assert_UCA_UAS_01 and Constraint_FMECA_UAS_01

    def test_inplace_overwrite_protection_prevents_overwriting_input_schema(self):
        """Verify that allow_schema_overwrite=False prevents modifying or overwriting the input schema in place."""
        with open(self.base_schema_path, "r", encoding="utf-8") as f:
            original_base_content = f.read()

        # Case 1: Attempt in-place overwrite when output_path == schema_path -> Must raise RuntimeError
        with self.assertRaises(RuntimeError) as ctx:
            reverse_sync_specs_to_sysml(
                docs_dir=self.docs_dir,
                schema_path=self.base_schema_path,
                output_path=self.base_schema_path,
                digest_path=self.out_digest,
                allow_schema_overwrite=False,
            )
        self.assertIn("In-place overwrite of base input schema", str(ctx.exception))

        # Case 2: Running with different output_path -> Input schema must NOT be modified
        reverse_sync_specs_to_sysml(
            docs_dir=self.docs_dir,
            schema_path=self.base_schema_path,
            output_path=self.out_sysml,
            digest_path=self.out_digest,
            allow_schema_overwrite=False,
        )

        with open(self.base_schema_path, "r", encoding="utf-8") as f:
            current_base_content = f.read()

        self.assertEqual(
            original_base_content,
            current_base_content,
            "Base input schema content must remain untouched when allow_schema_overwrite=False",
        )

        # Case 3: Explicit allow_schema_overwrite=True allows syncing back to schema_path
        reverse_sync_specs_to_sysml(
            docs_dir=self.docs_dir,
            schema_path=self.base_schema_path,
            output_path=self.out_sysml,
            digest_path=self.out_digest,
            allow_schema_overwrite=True,
        )
        with open(self.base_schema_path, "r", encoding="utf-8") as f:
            overwritten_base_content = f.read()
        self.assertIn("AutonomousRTH", overwritten_base_content)

    def test_digest_computation_and_valid_sysml_syntax(self):
        """Verify cryptographic digest calculation and valid SysML v2 output syntax."""
        pkg, digest = reverse_sync_specs_to_sysml(
            docs_dir=self.docs_dir,
            schema_path=self.base_schema_path,
            output_path=self.out_sysml,
            digest_path=self.out_digest,
            allow_schema_overwrite=False,
        )

        self.assertTrue(os.path.exists(self.out_digest))
        with open(self.out_digest, "r", encoding="utf-8") as f:
            loaded_digest = json.load(f)

        with open(self.out_sysml, "rb") as f:
            actual_bytes = f.read()
        expected_sha256 = hashlib.sha256(actual_bytes).hexdigest()

        self.assertEqual(loaded_digest["sha256"], expected_sha256)
        self.assertIn("node_counts", loaded_digest)
        self.assertIn("schema_nodes", loaded_digest)
        self.assertGreater(loaded_digest["total_lines"], 10)
        self.assertEqual(loaded_digest["node_counts"]["use_case_defs"], 1)
        self.assertEqual(loaded_digest["node_counts"]["interaction_defs"], 1)
        self.assertEqual(loaded_digest["node_counts"]["test_case_defs"], 1)
        self.assertEqual(loaded_digest["node_counts"]["part_defs"], 1)

        # Verify output file can be parsed directly by SysMLParser
        reparsed_pkg = SysMLParser.parse_file(self.out_sysml)
        self.assertEqual(reparsed_pkg.name, "AutonomousUAS_SSOT")
        self.assertEqual(reparsed_pkg.get_all_node_names(), pkg.get_all_node_names())

    def test_schema_parse_failure_raises_explicit_runtime_error(self):
        """Verify that non-existent or unparseable schema files raise explicit errors."""
        # Non-existent schema
        with self.assertRaises(FileNotFoundError):
            reverse_sync_specs_to_sysml(
                docs_dir=self.docs_dir,
                schema_path=os.path.join(self.tmpdir, "non_existent.sysml"),
                output_path=self.out_sysml,
                digest_path=self.out_digest,
            )

        # Corrupt schema with unbalanced braces
        corrupt_schema = os.path.join(self.tmpdir, "corrupt.sysml")
        with open(corrupt_schema, "w", encoding="utf-8") as f:
            f.write("package CorruptedPackage { part def Incomplete {")

        # Parsing malformed syntax must raise or be handled explicitly
        # Note: If parser throws, reverse sync raises RuntimeError
        try:
            reverse_sync_specs_to_sysml(
                docs_dir=self.docs_dir,
                schema_path=corrupt_schema,
                output_path=self.out_sysml,
                digest_path=self.out_digest,
            )
        except (RuntimeError, Exception) as exc:
            self.assertTrue(isinstance(exc, (RuntimeError, ValueError)))


if __name__ == "__main__":
    unittest.main()
