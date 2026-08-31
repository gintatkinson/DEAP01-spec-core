#!/usr/bin/env python3
"""
Unit tests for SysML v2 AST Grammar Expansion & Canonical Parser.

Verifies AST node representation, parser completeness, schema ingestion,
and digest serialization across all 6 Parity Matrix constructs:
1. Structural / Part / Package Definitions (SysMLPackage, PartDef)
2. Behavioral Actions & Operations (ActionDef, SysMLOperationDef)
3. System Capabilities (SysMLCapabilityDef)
4. Interaction Sequences & Flows (SysMLInteractionDef)
5. Constraints & Assertions (SysMLConstraintDef, assert constraint)
6. Verification Test Cases (SysMLTestCaseDef)
"""

import os
import sys
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
    SysMLPart,
    AttributeDef,
    PortDef,
    ActionDef,
    SysMLOperationDef,
    SysMLCapabilityDef,
    SysMLInteractionDef,
    SysMLConstraintDef,
    SysMLTestCaseDef,
    RequirementDef,
    StateDef,
    UseCaseDef,
    ItemDef,
    SysMLParser,
)
from sysmlv2_ingest import ingest_schema, detect_format


SAMPLE_COMPREHENSIVE_SYSML = """
package AutonomousUAS_SSOT {
    doc /* Single Source of Truth for Autonomous UAS Infrastructure Safety */

    attribute systemState : String = "STANDBY";
    inout port telemetryPort : TelemetryStream;

    capability def AutonomousCollisionAvoidance {
        doc /* Real-time detect-and-avoid capability */
        subsystem FlightGuidance;
        description: "Executes emergency trajectory deconfliction";
    }

    interaction TrajectoryDeconfliction {
        doc /* Coordination sequence during collision hazard */
        lifeline RadarSensor;
        lifeline GuidanceComputer;
        lifeline MotorController;
        message detectObstacle;
        message sendEmergencyVector;
        trigger proximityAlert;
        trigger geofenceBreach;
    }

    constraint def MaxVelocityLimit {
        doc /* Maximum airspeed safety envelope invariant */
        airspeed <= 35.0;
    }

    assert constraint SafeAltitudeConstraint {
        doc /* Ground collision avoidance assertion */
        altitude >= 50.0;
    }

    test case def TC_CollisionAvoidance_001 {
        doc /* Verification test case for DAA response latency */
        subject FlightGuidanceComputer;
        verify requirement REQ_SAF_001;
        objective "Verify avoidance vector computed in under 150ms";
        step inject_synthetic_threat;
        step assert_evasive_maneuver_commanded;
    }

    requirement def REQ_SAF_001 {
        id = "REQ-SAF-001";
        doc /* System must maintain minimum safe separation from dynamic obstacles */
        assume separationDistance > 10.0;
        require evasiveManeuverActive == true;
        verify by TC_CollisionAvoidance_001;
        satisfy by FlightGuidanceComputer;
    }

    state def FlightStatechart {
        doc /* UAS operational lifecycle state machine */
        entry initSensors;
        do monitorAirspace;
        exit safeShutdown;
        transition ArmedToInFlight;
    }

    use case def AutonomousRTH {
        doc /* Return to home on link loss */
        subject FlightController;
        actor GroundStationOperator;
        objective "Safely navigate UAS back to launch coordinates";
        include EmergencyFailsafe;
    }

    item def WaypointPayload {
        attribute latitude : Float;
        attribute longitude : Float;
        attribute altitude : Float;
    }

    part def FlightGuidanceComputer {
        doc /* Flight guidance and mission supervisory controller */
        attribute activeRouteId : String = "ROUTE_ALPHA";
        in port sensorStreamIn : SensorData;
        out port actuationCommandOut : ActuatorCommand;

        action ExecuteManeuver(in targetHeading : Float, out maneuverStatus : StatusEnum);

        operation def ComputeThrustVector(in currentHeading : Vector3D, in targetHeading : Vector3D) : ThrustVector;

        capability def PrecisionLanding {
            doc /* Autonomous beacon-guided landing */
            subsystem FlightGuidanceComputer;
        }

        interaction SensorFusionCycle {
            lifeline IMUSensor;
            lifeline FusionFilter;
            message sampleRate100Hz;
            trigger timestampSync;
        }

        assert constraint ThermalLimitConstraint {
            coreTemperature <= 85.0;
        }

        test case def TC_ThermalThrottling {
            subject FlightGuidanceComputer;
            verify requirement REQ_SAF_002;
            objective "Verify cpu throttling under high thermal load";
            step apply_thermal_stress;
        }
    }
}
"""


def test_sysmlv2_ast_dataclasses_instantiation_and_sysml_emission():
    """Verify programmatic instantiation and SysML textual emission of all 6 AST nodes."""
    cap = SysMLCapabilityDef(
        name="SenseAndAvoid",
        description="Autonomous DAA capability",
        subsystem="SurveillanceSubsystem",
        doc="Sense and avoid doc"
    )
    assert cap.name == "SenseAndAvoid"
    assert cap.subsystem == "SurveillanceSubsystem"
    assert "capability def SenseAndAvoid" in cap.to_sysml()

    op = SysMLOperationDef(
        name="CalculateTrajectory",
        direction="in",
        param_type="Waypoint",
        return_type="TrajectoryPlan",
        parameters=[
            AttributeDef(name="wp", type_name="Waypoint", default_value="in"),
            AttributeDef(name="speed", type_name="Float", default_value="in")
        ]
    )
    assert op.name == "CalculateTrajectory"
    assert "operation CalculateTrajectory" in op.to_sysml()

    inter = SysMLInteractionDef(
        name="HandshakeInteraction",
        lifelines=["UAV", "GCS"],
        messages=["reqConnect", "ackConnect"],
        triggers=["heartbeatTick"]
    )
    assert inter.name == "HandshakeInteraction"
    assert len(inter.lifelines) == 2
    assert "interaction HandshakeInteraction" in inter.to_sysml()

    c_def = SysMLConstraintDef(
        name="MaxPitchConstraint",
        expression="pitchAngle <= 45.0",
        is_assertion=False
    )
    assert c_def.name == "MaxPitchConstraint"
    assert not c_def.is_assertion
    assert "constraint def MaxPitchConstraint" in c_def.to_sysml()

    c_assert = SysMLConstraintDef(
        name="PositiveThrustAssertion",
        expression="thrust >= 0.0",
        is_assertion=True
    )
    assert c_assert.is_assertion
    assert "assert constraint PositiveThrustAssertion" in c_assert.to_sysml()

    tc = SysMLTestCaseDef(
        name="TC_RTH_LinkLoss",
        subject_part="FlightController",
        verified_requirements=["REQ-COM-001"],
        objective="Verify RTH triggers within 3 seconds of link loss",
        test_steps=["sever_c2_link", "assert_rth_mode_active"]
    )
    assert tc.name == "TC_RTH_LinkLoss"
    assert tc.subject_part == "FlightController"
    assert "test case def TC_RTH_LinkLoss" in tc.to_sysml()


def test_sysml_parser_comprehensive_model():
    """Verify SysMLParser parses a comprehensive SysML v2 model with all 6 constructs."""
    pkg = SysMLParser.parse_text(SAMPLE_COMPREHENSIVE_SYSML)

    assert isinstance(pkg, SysMLPackage)
    assert pkg.name == "AutonomousUAS_SSOT"

    # 1. Structural / Part Definitions
    assert len(pkg.part_defs) == 1
    part = pkg.part_defs[0]
    assert part.name == "FlightGuidanceComputer"
    assert isinstance(part, SysMLPart)

    # 2. Behavioral Actions & Operations
    assert len(part.actions) == 1
    action = part.actions[0]
    assert action.name == "ExecuteManeuver"
    assert len(action.in_params) == 1
    assert action.in_params[0].name == "targetHeading"
    assert len(action.out_params) == 1
    assert action.out_params[0].name == "maneuverStatus"

    assert len(part.operations) == 1
    op = part.operations[0]
    assert op.name == "ComputeThrustVector"
    assert op.return_type == "ThrustVector"
    assert len(op.parameters) == 2
    assert op.parameters[0].name == "currentHeading"
    assert op.parameters[1].name == "targetHeading"

    # 3. System Capabilities
    assert len(pkg.capability_defs) == 1
    cap = pkg.capability_defs[0]
    assert cap.name == "AutonomousCollisionAvoidance"
    assert cap.subsystem == "FlightGuidance"
    assert "deconfliction" in cap.description.lower() or "detect-and-avoid" in cap.description.lower()

    assert len(part.capabilities) == 1
    part_cap = part.capabilities[0]
    assert part_cap.name == "PrecisionLanding"
    assert part_cap.subsystem == "FlightGuidanceComputer"

    # 4. Interaction Sequences
    assert len(pkg.interaction_defs) == 1
    inter = pkg.interaction_defs[0]
    assert inter.name == "TrajectoryDeconfliction"
    assert "RadarSensor" in inter.lifelines
    assert "GuidanceComputer" in inter.lifelines
    assert "detectObstacle" in inter.messages
    assert "proximityAlert" in inter.triggers

    assert len(part.interactions) == 1
    part_inter = part.interactions[0]
    assert part_inter.name == "SensorFusionCycle"
    assert "IMUSensor" in part_inter.lifelines

    # 5. Constraints & Assertions
    assert len(pkg.constraint_defs) == 2
    c_names = {c.name: c for c in pkg.constraint_defs}
    assert "MaxVelocityLimit" in c_names
    assert not c_names["MaxVelocityLimit"].is_assertion
    assert "SafeAltitudeConstraint" in c_names
    assert c_names["SafeAltitudeConstraint"].is_assertion

    assert len(part.constraints) == 1
    assert part.constraints[0].name == "ThermalLimitConstraint"
    assert part.constraints[0].is_assertion

    # 6. Test Cases
    assert len(pkg.test_case_defs) == 1
    tc = pkg.test_case_defs[0]
    assert tc.name == "TC_CollisionAvoidance_001"
    assert tc.subject_part == "FlightGuidanceComputer"
    assert "REQ_SAF_001" in tc.verified_requirements
    assert "inject_synthetic_threat" in tc.test_steps

    assert len(part.test_cases) == 1
    part_tc = part.test_cases[0]
    assert part_tc.name == "TC_ThermalThrottling"
    assert part_tc.subject_part == "FlightGuidanceComputer"
    assert "REQ_SAF_002" in part_tc.verified_requirements

    # Additional model elements
    assert len(pkg.requirement_defs) == 1
    assert pkg.requirement_defs[0].name == "REQ_SAF_001"
    assert pkg.requirement_defs[0].req_id == "REQ-SAF-001"

    assert len(pkg.state_defs) == 1
    assert pkg.state_defs[0].name == "FlightStatechart"

    assert len(pkg.use_case_defs) == 1
    assert pkg.use_case_defs[0].name == "AutonomousRTH"

    assert len(pkg.item_defs) == 1
    assert pkg.item_defs[0].name == "WaypointPayload"


def test_sysml_package_node_counts_and_all_names():
    """Verify node_counts() and get_all_node_names() properly aggregate all 6 constructs."""
    pkg = SysMLParser.parse_text(SAMPLE_COMPREHENSIVE_SYSML)
    counts = pkg.node_counts()

    assert counts["packages"] == 1
    assert counts["part_defs"] == 1
    assert counts["attribute_defs"] >= 2
    assert counts["port_defs"] >= 3
    assert counts["action_defs"] >= 1
    assert counts["capability_defs"] == 2  # 1 package-level + 1 part-level
    assert counts["operation_defs"] == 1   # 1 part-level
    assert counts["interaction_defs"] == 2 # 1 package-level + 1 part-level
    assert counts["constraint_defs"] == 3  # 2 package-level + 1 part-level
    assert counts["test_case_defs"] == 2   # 1 package-level + 1 part-level
    assert counts["requirement_defs"] == 1
    assert counts["state_defs"] == 1
    assert counts["use_case_defs"] == 1
    assert counts["item_defs"] == 1

    all_names = pkg.get_all_node_names()
    expected_names = [
        "AutonomousUAS_SSOT",
        "FlightGuidanceComputer",
        "AutonomousCollisionAvoidance",
        "PrecisionLanding",
        "ComputeThrustVector",
        "ExecuteManeuver",
        "TrajectoryDeconfliction",
        "SensorFusionCycle",
        "MaxVelocityLimit",
        "SafeAltitudeConstraint",
        "ThermalLimitConstraint",
        "TC_CollisionAvoidance_001",
        "TC_ThermalThrottling",
        "REQ_SAF_001",
        "FlightStatechart",
        "AutonomousRTH",
        "WaypointPayload"
    ]
    for expected in expected_names:
        assert expected in all_names, f"Expected node name '{expected}' missing from get_all_node_names()"


def test_sysmlv2_ingest_pipeline_digest_generation():
    """Verify sysmlv2_ingest generates valid schema-digest.json capturing all 6 constructs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        input_sysml = os.path.join(tmpdir, "model.sysml")
        out_sysml = os.path.join(tmpdir, "out_schema.sysml")
        out_digest = os.path.join(tmpdir, "schema-digest.json")

        with open(input_sysml, "w", encoding="utf-8") as f:
            f.write(SAMPLE_COMPREHENSIVE_SYSML)

        assert detect_format(input_sysml, SAMPLE_COMPREHENSIVE_SYSML) == "sysml"

        pkg, digest = ingest_schema(
            schema_path=input_sysml,
            format_type="sysml",
            output_path=out_sysml,
            digest_path=out_digest
        )

        assert os.path.isfile(out_digest)
        assert "node_counts" in digest
        assert "schema_nodes" in digest
        assert digest["node_counts"]["capability_defs"] == 2
        assert digest["node_counts"]["operation_defs"] == 1
        assert digest["node_counts"]["interaction_defs"] == 2
        assert digest["node_counts"]["constraint_defs"] == 3
        assert digest["node_counts"]["test_case_defs"] == 2
        assert "AutonomousCollisionAvoidance" in digest["schema_nodes"]
        assert "ComputeThrustVector" in digest["schema_nodes"]
        assert "TrajectoryDeconfliction" in digest["schema_nodes"]
        assert "MaxVelocityLimit" in digest["schema_nodes"]
        assert "TC_CollisionAvoidance_001" in digest["schema_nodes"]


def test_compile_sysml_integration():
    """Verify scripts/compile_sysml.py extracts all 6 AST node types."""
    import importlib.util
    compile_path = os.path.join(PROJECT_ROOT, "scripts", "compile_sysml.py")
    spec = importlib.util.spec_from_file_location("compile_sysml", compile_path)
    compile_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(compile_module)

    ast_dict = compile_module.parse_sysml(SAMPLE_COMPREHENSIVE_SYSML)

    assert "AutonomousUAS_SSOT" in ast_dict["packages"]
    assert "FlightGuidanceComputer" in ast_dict["part_defs"]
    assert "AutonomousCollisionAvoidance" in ast_dict["capability_defs"]
    assert "PrecisionLanding" in ast_dict["capability_defs"]
    assert "ComputeThrustVector" in ast_dict["operation_defs"]
    assert "TrajectoryDeconfliction" in ast_dict["interaction_defs"]
    assert "MaxVelocityLimit" in ast_dict["constraint_defs"]
    assert "SafeAltitudeConstraint" in ast_dict["constraint_defs"]
    assert "ThermalLimitConstraint" in ast_dict["constraint_defs"]
    assert "TC_CollisionAvoidance_001" in ast_dict["test_case_defs"]
    assert "TC_ThermalThrottling" in ast_dict["test_case_defs"]


def test_sysml_parser_roundtrip():
    """Verify model -> to_sysml() -> re-parse roundtrip stability."""
    pkg1 = SysMLParser.parse_text(SAMPLE_COMPREHENSIVE_SYSML)
    emitted_sysml = pkg1.to_sysml()

    pkg2 = SysMLParser.parse_text(emitted_sysml)

    assert pkg2.name == pkg1.name
    assert len(pkg2.part_defs) == len(pkg1.part_defs)
    assert len(pkg2.capability_defs) == len(pkg1.capability_defs)
    assert len(pkg2.interaction_defs) == len(pkg1.interaction_defs)
    assert len(pkg2.constraint_defs) == len(pkg1.constraint_defs)
    assert len(pkg2.test_case_defs) == len(pkg1.test_case_defs)
    assert pkg2.get_all_node_names() == pkg1.get_all_node_names()


def test_compile_sysml_stpa_sanitized_assertions():
    """Verify compile_sysml STPA synthesis emits sanitized domain-agnostic AST assertions."""
    import importlib.util
    compile_path = os.path.join(PROJECT_ROOT, "scripts", "compile_sysml.py")
    spec = importlib.util.spec_from_file_location("compile_sysml", compile_path)
    compile_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(compile_module)

    # Test generic fallback sanitization
    sample_text = "# STPA Section\n- Item UCA-01: Critical system event"
    ucas = compile_module.parse_stpa_ucas(sample_text)
    assert len(ucas) == 1
    assert ucas[0]["controller"] == "SafetyController"
    assert ucas[0]["control_action"] == "SystemSafetyAction"
    assert ucas[0]["context"] == "OperationalBoundExceeded"
    assert ucas[0]["hazard"] == "H_System_Hazard"
    assert ucas[0]["severity"] == "Critical"
    assert ucas[0]["sail"] == "SafetyLevel_High"

    c = compile_module.compile_uca_to_constraint(ucas[0])
    assert c.name == "Assert_UCA_01"
    assert c.expression == "systemParameter <= maxThreshold"

    # Test timeout/loss predicate synthesis
    uca_loss = {
        "id": "UCA-02",
        "controller": "SafetyController",
        "control_action": "SystemSafetyAction",
        "context": "t_loss > 2.0 s, Link Timeout",
        "hazard": "H_System_Hazard",
        "severity": "Critical",
        "sail": "SafetyLevel_High"
    }
    c_loss = compile_module.compile_uca_to_constraint(uca_loss)
    assert c_loss.name == "Assert_UCA_02"
    assert c_loss.expression == "lossDuration <= timeoutLimit"
