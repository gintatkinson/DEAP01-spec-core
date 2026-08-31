#!/usr/bin/env python3
"""
Unit tests for SysML v2 Closed-Loop Reverse Synchronization Engine (`compile_sysml.py --reverse-sync`).

Verifies:
1. Reverse extraction of Use Cases into `UseCaseDef` and SysML textual emission.
2. Reverse extraction of User Story lifelines and BDD tests into `SysMLInteractionDef` and `SysMLTestCaseDef`.
3. Reverse extraction of Feature operations into `SysMLOperationDef` and `ActionDef` with typed parameters.
4. Reverse extraction of STPA hazards into `SysMLConstraintDef` assertions.
5. End-to-end `compile_sysml.py --reverse-sync` CLI execution on a sample docs hierarchy.
6. Idempotent round-trip synchronization (Diff(AST_1, AST_2) == 0).
"""

import os
import sys
import json
import subprocess
import tempfile
import pytest

# Ensure spec-orchestrator scripts and parity_auditor are on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
SPEC_ORCH_DIR = os.path.abspath(os.path.join(PARITY_AUDITOR_DIR, ".."))

def find_repo_root(start_dir: str) -> str:
    cur = os.path.abspath(start_dir)
    while cur and cur != os.path.dirname(cur):
        if os.path.exists(os.path.join(cur, "scripts", "compile_sysml.py")):
            return cur
        cur = os.path.dirname(cur)
    return os.path.abspath(os.path.join(start_dir, "..", "..", "..", ".."))

PROJECT_ROOT = find_repo_root(SCRIPT_DIR)
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
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
    RequirementDef,
    StateDef,
    ItemDef,
    SysMLParser,
)
from scripts.compile_sysml import (
    extract_use_cases_from_markdown,
    extract_user_story_ast,
    extract_features_from_markdown,
    extract_epics_from_markdown,
    parse_stpa_ucas,
    parse_fmeca_modes,
    compile_uca_to_constraint,
    compile_fmeca_to_constraint,
    compile_stpa_to_ast,
    compile_stpa_to_sysml,
    reverse_sync_specs_to_sysml,
)


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

## 2. Alternate Flows
### 1. Link Recovery
1. System monitors C2 link reconnect.
2. System transitions back to nominal navigation.
3. System resumes mission plan.
"""

SAMPLE_USER_STORY_MD = """---
title: "Autonomous Trajectory Deconfliction"
type: "user-story"
generation_mode: "subagent"
interaction: "TrajectoryDeconfliction"
test_case: "TC_CollisionAvoidance_001"
---

# User Story: Autonomous Trajectory Deconfliction

## BDD Scenario (OOA/OOD Realization)
**Given** UAS is maintaining active cruise trajectory
**When** Proximity alert is received from radar
**Then** Flight guidance executes emergency deconfliction maneuver

## UML Sequence Diagram
```mermaid
sequenceDiagram
    autonumber
    actor radarOperator as "radarOperator : RadarOperator"
    participant flightGuidance as "flightGuidance : FlightGuidanceComputer"
    participant motorController as "motorController : MotorController"

    radarOperator->>flightGuidance: ExecuteManeuver(targetHeading: Float)
    flightGuidance->>motorController: sendEmergencyVector(vector: Vector3D)
    flightGuidance-->radarOperator: status : Status
```

## Stateflow Transition Triggers
- trigger proximityAlert
- trigger geofenceBreach

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `TC_CollisionAvoidance_001`
- **Subject Part:** `FlightGuidanceComputer`
- **Verified Safety Requirement:** `REQ_SAF_001`
- **Verification Objective:** "Verify emergency vector computed within 150ms"
- **Test Steps:**
  - `step inject_synthetic_threat`
  - `step assert_evasive_maneuver_commanded`
"""

SAMPLE_FEATURE_MD = """---
title: "Flight Guidance Computer"
type: "feature"
interface_type: "m2m"
generation_mode: "subagent"
spec_source: "Project Constitution"
schema_containers:
  - path: "AutonomousUAS_SSOT:AutonomousUAS_SSOT/FlightGuidanceComputer"
    node_type: container
---

# Feature: Flight Guidance Computer

## UML Class Diagram
```mermaid
classDiagram
    class FlightGuidanceComputer {
        +String activeRouteId "[1]"
        +Boolean ExecuteManeuver(Float in_targetHeading, StatusEnum out_maneuverStatus)
        +ThrustVector ComputeThrustVector(Vector3D in_currentHeading, Vector3D in_targetHeading)
    }
```

## Interface Requirements

### 1. Payload Schema
```json
{
  "activeRouteId": "ROUTE_ALPHA"
}
```

### 2. Validation & Constraints
- ThermalLimitConstraint: coreTemperature <= 85.0

### 3. Logical Operations & Interface Messages
- `+ExecuteManeuver(in targetHeading : Float, out maneuverStatus : StatusEnum)`: Executes emergency evasive deconfliction maneuver.
- `+ComputeThrustVector(in currentHeading : Vector3D, in targetHeading : Vector3D) : ThrustVector`: Calculates required 3D thrust vector.

### 4. Logical Exception States & Validation Failures
- Thermal throttling triggers on coreTemperature > 85.0.

## Given-When-Then Acceptance Criteria
- Given flight guidance computer in navigation mode, When obstacle detected, Then ExecuteManeuver commands trajectory change.
"""

SAMPLE_STPA_MATRIX_MD = """# STPA Unsafe Control Action (UCA) Matrix

| UCA ID | Controller | Control Action | STPA UCA Category | Environmental Context Vector | Triggered System Hazard | Severity Classification | SORA SAIL Target |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCA-UAS-01** | Flight Guidance Computer | Fail-Safe Return-to-Launch (RTL) | **1. Not Provided** | $t_{\\text{loss}} > 2.0\\text{ s}$, $d_{\\text{geo}} < 15\\text{ m}$, C2 Link Down | **H_UAS_1:** Lost-Link Flyaway / Airspace Infringement | Catastrophic | **SAIL IV–VI** |
| **UCA-UAS-02** | Motor ESC | Emergency Rotor Braking | **2. Provided Unsafely** | $h_{\\text{AGL}} > 50\\text{ m}$, high speed flight | **H_UAS_2:** Loss of Altitude Control | Critical | **SAIL IV** |

## FMECA Failure Modes
| FMECA ID | Component | Failure Mode | Effect | Safety Invariant |
| :--- | :--- | :--- | :--- | :--- |
| **FMECA-UAS-01** | Magnetometer | Magnetic flux saturation | Compass heading divergence | flux <= 250.0 uT |
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


def test_reverse_extraction_use_cases_to_usecase_def_and_sysml_emission():
    """Verify reverse extraction of Use Case specs into UseCaseDef AST nodes and textual emission."""
    use_cases = extract_use_cases_from_markdown(SAMPLE_USE_CASE_MD, filename="uc-01-autonomous-rth.md")
    assert len(use_cases) == 1
    uc = use_cases[0]

    assert isinstance(uc, UseCaseDef)
    assert uc.name == "AutonomousRTH"
    assert uc.subject == "FlightController"
    assert uc.actor == "GroundStationOperator"
    assert "Safely navigate UAS" in uc.objective
    assert "EmergencyFailsafe" in uc.includes
    assert "ManualOverride" in uc.extends

    sysml_text = uc.to_sysml()
    assert "use case def AutonomousRTH {" in sysml_text
    assert "subject FlightController;" in sysml_text
    assert "actor GroundStationOperator;" in sysml_text
    assert "objective \"Safely navigate UAS back to launch coordinates on C2 link loss\";" in sysml_text
    assert "include EmergencyFailsafe;" in sysml_text
    assert "extend ManualOverride;" in sysml_text

    # Re-parse into AST
    wrapper_sysml = f"package TestPackage {{\n{sysml_text}\n}}"
    pkg = SysMLParser.parse_text(wrapper_sysml)
    assert len(pkg.use_case_defs) == 1
    parsed_uc = pkg.use_case_defs[0]
    assert parsed_uc.name == "AutonomousRTH"
    assert parsed_uc.subject == "FlightController"
    assert parsed_uc.actor == "GroundStationOperator"
    assert "EmergencyFailsafe" in parsed_uc.includes
    assert "ManualOverride" in parsed_uc.extends


def test_reverse_extraction_user_story_lifelines_and_bdd_tests():
    """Verify reverse extraction of User Story lifelines and BDD tests into SysMLInteractionDef and SysMLTestCaseDef."""
    interactions, test_cases = extract_user_story_ast(SAMPLE_USER_STORY_MD, filename="us-01-trajectory-deconfliction.md")

    # 1. InteractionDef
    assert len(interactions) == 1
    inter = interactions[0]
    assert isinstance(inter, SysMLInteractionDef)
    assert inter.name == "TrajectoryDeconfliction"
    assert "RadarOperator" in inter.lifelines
    assert "FlightGuidanceComputer" in inter.lifelines
    assert "MotorController" in inter.lifelines
    assert "ExecuteManeuver" in inter.messages
    assert "sendEmergencyVector" in inter.messages
    assert "proximityAlert" in inter.triggers
    assert "geofenceBreach" in inter.triggers

    inter_sysml = inter.to_sysml()
    assert "interaction TrajectoryDeconfliction {" in inter_sysml
    assert "lifeline RadarOperator;" in inter_sysml
    assert "message ExecuteManeuver;" in inter_sysml
    assert "trigger proximityAlert;" in inter_sysml

    # 2. SysMLTestCaseDef
    assert len(test_cases) == 1
    tc = test_cases[0]
    assert isinstance(tc, SysMLTestCaseDef)
    assert tc.name == "TC_CollisionAvoidance_001"
    assert tc.subject_part == "FlightGuidanceComputer"
    assert "REQ_SAF_001" in tc.verified_requirements
    assert "150ms" in tc.objective
    assert "inject_synthetic_threat" in tc.test_steps
    assert "assert_evasive_maneuver_commanded" in tc.test_steps

    tc_sysml = tc.to_sysml()
    assert "test case def TC_CollisionAvoidance_001 {" in tc_sysml
    assert "subject FlightGuidanceComputer;" in tc_sysml
    assert "verify requirement REQ_SAF_001;" in tc_sysml
    assert "objective \"Verify emergency vector computed within 150ms\";" in tc_sysml
    assert "step inject_synthetic_threat;" in tc_sysml
    assert "step assert_evasive_maneuver_commanded;" in tc_sysml

    # Re-parse into AST
    wrapper_sysml = f"package TestPackage {{\n{inter_sysml}\n{tc_sysml}\n}}"
    pkg = SysMLParser.parse_text(wrapper_sysml)
    assert len(pkg.interaction_defs) == 1
    assert len(pkg.test_case_defs) == 1
    assert pkg.interaction_defs[0].name == "TrajectoryDeconfliction"
    assert pkg.test_case_defs[0].name == "TC_CollisionAvoidance_001"


def test_reverse_extraction_feature_operations_and_typed_parameters():
    """Verify reverse extraction of Feature operations into SysMLOperationDef and ActionDef with typed parameters."""
    parts = extract_features_from_markdown(SAMPLE_FEATURE_MD, filename="feat-01-flight-guidance-computer.md")
    assert len(parts) == 1
    part = parts[0]

    assert isinstance(part, PartDef)
    assert part.name == "FlightGuidanceComputer"

    # Attributes
    attr_names = {a.name: a for a in part.attributes}
    assert "activeRouteId" in attr_names
    assert attr_names["activeRouteId"].type_name == "String"

    # ActionDefs
    action_names = {a.name: a for a in part.actions}
    assert "ExecuteManeuver" in action_names
    act = action_names["ExecuteManeuver"]
    assert len(act.in_params) == 1
    assert act.in_params[0].name == "targetHeading"
    assert act.in_params[0].type_name == "Float"
    assert len(act.out_params) == 1
    assert act.out_params[0].name == "maneuverStatus"
    assert act.out_params[0].type_name == "StatusEnum"

    # SysMLOperationDefs
    op_names = {o.name: o for o in part.operations}
    assert "ComputeThrustVector" in op_names
    op = op_names["ComputeThrustVector"]
    assert op.return_type == "ThrustVector"
    assert len(op.parameters) == 2
    assert op.parameters[0].name == "currentHeading"
    assert op.parameters[0].type_name == "Vector3D"
    assert op.parameters[1].name == "targetHeading"
    assert op.parameters[1].type_name == "Vector3D"

    # Validation Constraints
    con_names = {c.name: c for c in part.constraints}
    assert "ThermalLimitConstraint" in con_names
    assert con_names["ThermalLimitConstraint"].expression == "coreTemperature <= 85.0"
    assert con_names["ThermalLimitConstraint"].is_assertion is True

    # Serialized SysML verification
    part_sysml = part.to_sysml()
    assert "part def FlightGuidanceComputer {" in part_sysml
    assert "attribute activeRouteId : String;" in part_sysml
    assert "action ExecuteManeuver(in targetHeading : Float, out maneuverStatus : StatusEnum);" in part_sysml
    assert "operation ComputeThrustVector(in currentHeading : Vector3D, in targetHeading : Vector3D) : ThrustVector;" in part_sysml
    assert "assert constraint ThermalLimitConstraint {" in part_sysml


def test_reverse_extraction_stpa_hazards_to_sysml_assertions():
    """Verify reverse extraction of STPA hazards into SysMLConstraintDef assertions."""
    ucas = parse_stpa_ucas(SAMPLE_STPA_MATRIX_MD)
    assert len(ucas) == 2

    c1 = compile_uca_to_constraint(ucas[0])
    assert c1.name == "Assert_UCA_UAS_01"
    assert c1.is_assertion is True
    assert "lossDuration <= timeoutLimit" in c1.expression

    c2 = compile_uca_to_constraint(ucas[1])
    assert c2.name == "Assert_UCA_UAS_02"
    assert c2.is_assertion is True
    assert "systemParameter <= maxThreshold" in c2.expression

    fmecas = parse_fmeca_modes(SAMPLE_STPA_MATRIX_MD)
    assert len(fmecas) == 1
    f1 = compile_fmeca_to_constraint(fmecas[0])
    assert f1.name == "Constraint_FMECA_UAS_01"
    assert f1.is_assertion is False
    assert "Magnetometer_healthStatus == Normal" in f1.expression

    pkg = compile_stpa_to_ast(SAMPLE_STPA_MATRIX_MD)
    assert len(pkg.constraint_defs) == 3
    sysml_text = pkg.to_sysml()
    assert "package AutonomousUAS_SafetyConstraints {" in sysml_text
    assert "assert constraint Assert_UCA_UAS_01 {" in sysml_text
    assert "assert constraint Assert_UCA_UAS_02 {" in sysml_text
    assert "constraint def Constraint_FMECA_UAS_01 {" in sysml_text


def test_parse_stpa_ucas_fallback_sanitization():
    """Verify parse_stpa_ucas generic fallback synthesizes abstract domain-agnostic MBSE identifiers."""
    generic_content = "# Section\n- Item UCA-99: Generic unclassified control action"
    ucas = parse_stpa_ucas(generic_content)
    assert len(ucas) == 1
    uca = ucas[0]
    assert uca["id"] == "UCA-99"
    assert uca["controller"] == "SafetyController"
    assert uca["control_action"] == "SystemSafetyAction"
    assert uca["context"] == "OperationalBoundExceeded"
    assert uca["hazard"] == "H_System_Hazard"
    assert uca["severity"] == "Critical"
    assert uca["sail"] == "SafetyLevel_High"

    c = compile_uca_to_constraint(uca)
    assert c.name == "Assert_UCA_99"
    assert c.is_assertion is True
    assert c.expression == "systemParameter <= maxThreshold"


def test_end_to_end_compile_sysml_reverse_sync_cli():
    """Verify end-to-end compile_sysml.py --reverse-sync CLI execution on a sample docs hierarchy."""
    with tempfile.TemporaryDirectory() as tmpdir:
        docs_dir = os.path.join(tmpdir, "docs")
        uc_dir = os.path.join(docs_dir, "use-cases")
        us_dir = os.path.join(docs_dir, "user-stories")
        feat_dir = os.path.join(docs_dir, "features")
        epic_dir = os.path.join(docs_dir, "epics")
        safety_dir = os.path.join(docs_dir, "safety")
        os.makedirs(uc_dir, exist_ok=True)
        os.makedirs(us_dir, exist_ok=True)
        os.makedirs(feat_dir, exist_ok=True)
        os.makedirs(epic_dir, exist_ok=True)
        os.makedirs(safety_dir, exist_ok=True)

        with open(os.path.join(uc_dir, "uc-01-autonomous-rth.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USE_CASE_MD)
        with open(os.path.join(us_dir, "us-01-trajectory-deconfliction.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USER_STORY_MD)
        with open(os.path.join(feat_dir, "feat-01-flight-guidance-computer.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_FEATURE_MD)
        with open(os.path.join(epic_dir, "epic-01-flight-guidance.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_EPIC_MD)
        with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_STPA_MATRIX_MD)

        out_sysml = os.path.join(tmpdir, ".pipeline", "schema.sysml")
        out_digest = os.path.join(tmpdir, ".pipeline", "schema-digest.json")

        cmd = [
            sys.executable,
            os.path.join(PROJECT_ROOT, "scripts", "compile_sysml.py"),
            "--reverse-sync",
            "--docs", docs_dir,
            "--out", out_sysml,
            "--digest", out_digest
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        assert res.returncode == 0, f"CLI failed with error: {res.stderr}\nstdout: {res.stdout}"

        assert os.path.exists(out_sysml)
        assert os.path.exists(out_digest)

        with open(out_digest, "r", encoding="utf-8") as f:
            digest = json.load(f)

        assert "node_counts" in digest
        assert "schema_nodes" in digest
        assert digest["node_counts"]["use_case_defs"] >= 1
        assert digest["node_counts"]["interaction_defs"] >= 1
        assert digest["node_counts"]["test_case_defs"] >= 1
        assert digest["node_counts"]["part_defs"] >= 1
        assert digest["node_counts"]["constraint_defs"] >= 3
        assert digest["node_counts"]["capability_defs"] >= 2

        # Verify parsed schema contains all entities
        pkg = SysMLParser.parse_file(out_sysml)
        assert pkg.name == "AutonomousUAS_SSOT"
        assert len(pkg.use_case_defs) == 1
        assert len(pkg.interaction_defs) == 1
        assert len(pkg.test_case_defs) == 1
        assert len(pkg.part_defs) == 1
        assert len(pkg.capability_defs) == 2


def test_idempotent_roundtrip_synchronization():
    """Verify idempotent round-trip synchronization (Diff(AST_1, AST_2) == 0)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        docs_dir = os.path.join(tmpdir, "docs")
        uc_dir = os.path.join(docs_dir, "use-cases")
        us_dir = os.path.join(docs_dir, "user-stories")
        feat_dir = os.path.join(docs_dir, "features")
        epic_dir = os.path.join(docs_dir, "epics")
        safety_dir = os.path.join(docs_dir, "safety")
        os.makedirs(uc_dir, exist_ok=True)
        os.makedirs(us_dir, exist_ok=True)
        os.makedirs(feat_dir, exist_ok=True)
        os.makedirs(epic_dir, exist_ok=True)
        os.makedirs(safety_dir, exist_ok=True)

        with open(os.path.join(uc_dir, "uc-01-autonomous-rth.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USE_CASE_MD)
        with open(os.path.join(us_dir, "us-01-trajectory-deconfliction.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_USER_STORY_MD)
        with open(os.path.join(feat_dir, "feat-01-flight-guidance-computer.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_FEATURE_MD)
        with open(os.path.join(epic_dir, "epic-01-flight-guidance.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_EPIC_MD)
        with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
            f.write(SAMPLE_STPA_MATRIX_MD)

        out_sysml = os.path.join(tmpdir, ".pipeline", "schema.sysml")
        out_digest = os.path.join(tmpdir, ".pipeline", "schema-digest.json")

        # Pass 1
        pkg_1, digest_1 = reverse_sync_specs_to_sysml(
            docs_dir=docs_dir,
            output_path=out_sysml,
            digest_path=out_digest
        )
        with open(out_sysml, "r", encoding="utf-8") as f:
            sysml_pass1 = f.read()

        # Pass 2 (run reverse sync on top of Pass 1 output)
        pkg_2, digest_2 = reverse_sync_specs_to_sysml(
            docs_dir=docs_dir,
            schema_path=out_sysml,
            output_path=out_sysml,
            digest_path=out_digest
        )
        with open(out_sysml, "r", encoding="utf-8") as f:
            sysml_pass2 = f.read()

        # Pass 3
        pkg_3, digest_3 = reverse_sync_specs_to_sysml(
            docs_dir=docs_dir,
            schema_path=out_sysml,
            output_path=out_sysml,
            digest_path=out_digest
        )
        with open(out_sysml, "r", encoding="utf-8") as f:
            sysml_pass3 = f.read()

        # Verify exact AST & textual identity across round-trips
        assert sysml_pass1 == sysml_pass2 == sysml_pass3
        assert digest_1["sha256"] == digest_2["sha256"] == digest_3["sha256"]
        assert digest_1["node_counts"] == digest_2["node_counts"] == digest_3["node_counts"]
        assert digest_1["schema_nodes"] == digest_2["schema_nodes"] == digest_3["schema_nodes"]
        assert pkg_1.get_all_node_names() == pkg_2.get_all_node_names() == pkg_3.get_all_node_names()
