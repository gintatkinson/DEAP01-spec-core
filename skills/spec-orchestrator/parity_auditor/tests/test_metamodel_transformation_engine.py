#!/usr/bin/env python3
"""
Tests for Tier-1 Metamodel Transformation Engine (Issue #266).

Verifies:
1. ALLOWED_M2_METAMODEL_TYPES completeness and integrity.
2. is_allowed_m2_type resolution including case-insensitivity and meta_ prefixes.
3. map_ast_classifier_to_m2 mappings from AST classifiers and domain roles to Tier-1 M2 metamodel types.
4. validate_metamodel_purity detection of M1 domain typing violations and static parameter dicts.
5. MetamodelTransformationEngine integration as an IValidator.
"""

import ast
import unittest
from typing import Optional

from parity_auditor.validators.metamodel_transformation_engine import (
    ALLOWED_M2_METAMODEL_TYPES,
    MetamodelTransformationEngine,
    is_allowed_m2_type,
    map_ast_classifier_to_m2,
    validate_metamodel_purity,
)
from parity_auditor.core.findings import Finding


class TestMetamodelTransformationEngine(unittest.TestCase):
    """Test suite for MetamodelTransformationEngine and Tier-1 M2 metamodel typing."""

    def test_allowed_m2_metamodel_types_completeness(self):
        """Verify ALLOWED_M2_METAMODEL_TYPES contains all Tier-1 abstract metamodel entities."""
        mandatory_types = {
            "Component",
            "Class",
            "Port",
            "Interface",
            "Statechart",
            "Constraint",
            "ActorDefinition",
            "PartDefinition",
            "PortDefinition",
            "StateDefinition",
            "ActionDefinition",
            "ItemDefinition",
            "RequirementDefinition",
            "UseCaseDefinition",
            "ConstraintDefinition",
            "HumanOperator",
            "OperatorConsole",
            "SystemController",
            "SafetyInterlock",
            "PhysicalActuator",
            "Sensor",
            "SystemUnderStudy",
            "ExternalSystem",
            "LogicalUI",
            "Widget",
            "Signal",
            "Event",
            "AcceptanceCriterion",
            "Scenario",
            "TraceLink",
        }
        for m_type in mandatory_types:
            self.assertIn(m_type, ALLOWED_M2_METAMODEL_TYPES)
            self.assertTrue(is_allowed_m2_type(m_type), f"Expected {m_type} to be allowed M2 type")

    def test_is_allowed_m2_type_variations(self):
        """Verify is_allowed_m2_type handles normalization, case variations, and meta_ prefixes."""
        # Exact and normalized
        self.assertTrue(is_allowed_m2_type("LogicalUI"))
        self.assertTrue(is_allowed_m2_type("logical_ui"))
        self.assertTrue(is_allowed_m2_type("logical-ui"))
        self.assertTrue(is_allowed_m2_type("HumanOperator"))
        self.assertTrue(is_allowed_m2_type("human_operator"))
        self.assertTrue(is_allowed_m2_type("SystemController"))
        self.assertTrue(is_allowed_m2_type("system_controller"))
        self.assertTrue(is_allowed_m2_type("SafetyInterlock"))
        self.assertTrue(is_allowed_m2_type("safety_interlock"))
        self.assertTrue(is_allowed_m2_type("PhysicalActuator"))
        self.assertTrue(is_allowed_m2_type("physical_actuator"))
        self.assertTrue(is_allowed_m2_type("Sensor"))
        self.assertTrue(is_allowed_m2_type("sensor"))
        self.assertTrue(is_allowed_m2_type("Widget"))
        self.assertTrue(is_allowed_m2_type("widget"))

        # Meta prefixes
        self.assertTrue(is_allowed_m2_type("meta_custom_element"))
        self.assertTrue(is_allowed_m2_type("meta_flight_envelope"))
        self.assertTrue(is_allowed_m2_type("MetaEntity"))
        self.assertTrue(is_allowed_m2_type("meta-block"))

        # Engine static/instance method
        engine = MetamodelTransformationEngine()
        self.assertTrue(engine.is_allowed_m2_type("Component"))
        self.assertTrue(MetamodelTransformationEngine.is_allowed_m2_type("Component"))

        # Invalid M1 domain tokens
        self.assertFalse(is_allowed_m2_type("FlightController"))
        self.assertFalse(is_allowed_m2_type("AltimeterSensor"))
        self.assertFalse(is_allowed_m2_type("ElevatorServo"))
        self.assertFalse(is_allowed_m2_type("QuadRotorDrone"))
        self.assertFalse(is_allowed_m2_type(""))
        self.assertFalse(is_allowed_m2_type("   "))
        self.assertFalse(is_allowed_m2_type(None))  # type: ignore

    def test_map_ast_classifier_to_m2_with_domain_roles(self):
        """Verify map_ast_classifier_to_m2 correctly maps classifiers given domain roles."""
        engine = MetamodelTransformationEngine()

        self.assertEqual(engine.map_ast_classifier_to_m2("BarometricAltimeter", domain_role="sensor"), "Sensor")
        self.assertEqual(map_ast_classifier_to_m2("ElevatorActuator", domain_role="actuator"), "PhysicalActuator")
        self.assertEqual(map_ast_classifier_to_m2("RuddervatorServo", domain_role="physical_actuator"), "PhysicalActuator")
        self.assertEqual(map_ast_classifier_to_m2("GuidanceUnit", domain_role="controller"), "SystemController")
        self.assertEqual(map_ast_classifier_to_m2("AutopilotComputer", domain_role="system_controller"), "SystemController")
        self.assertEqual(map_ast_classifier_to_m2("FlightTerminationInterlock", domain_role="interlock"), "SafetyInterlock")
        self.assertEqual(map_ast_classifier_to_m2("EmergencyCutoff", domain_role="safety_interlock"), "SafetyInterlock")
        self.assertEqual(map_ast_classifier_to_m2("MissionPilot", domain_role="operator"), "HumanOperator")
        self.assertEqual(map_ast_classifier_to_m2("GroundControlStation", domain_role="console"), "OperatorConsole")
        self.assertEqual(map_ast_classifier_to_m2("PrimaryFlightDisplay", domain_role="ui"), "LogicalUI")
        self.assertEqual(map_ast_classifier_to_m2("AttitudeIndicator", domain_role="widget"), "Widget")
        self.assertEqual(map_ast_classifier_to_m2("PayloadBay", domain_role="part"), "PartDefinition")
        self.assertEqual(map_ast_classifier_to_m2("TelemetryPort", domain_role="port"), "PortDefinition")
        self.assertEqual(map_ast_classifier_to_m2("TelemetryPacket", domain_role="item"), "ItemDefinition")
        self.assertEqual(map_ast_classifier_to_m2("ArmedState", domain_role="state"), "StateDefinition")
        self.assertEqual(map_ast_classifier_to_m2("ComputeTrajectory", domain_role="action"), "ActionDefinition")
        self.assertEqual(map_ast_classifier_to_m2("SafeSeparationRequirement", domain_role="requirement"), "RequirementDefinition")
        self.assertEqual(map_ast_classifier_to_m2("PreFlightCheckUseCase", domain_role="usecase"), "UseCaseDefinition")

    def test_map_ast_classifier_to_m2_inference_without_domain_role(self):
        """Verify map_ast_classifier_to_m2 infers M2 type from classifier name tokens."""
        self.assertEqual(map_ast_classifier_to_m2("Component"), "Component")
        self.assertEqual(map_ast_classifier_to_m2("PartDefinition"), "PartDefinition")
        self.assertEqual(map_ast_classifier_to_m2("PitotStaticSensor"), "Sensor")
        self.assertEqual(map_ast_classifier_to_m2("RudderServoActuator"), "PhysicalActuator")
        self.assertEqual(map_ast_classifier_to_m2("FlightManagementController"), "SystemController")
        self.assertEqual(map_ast_classifier_to_m2("ArmingSafetyInterlock"), "SafetyInterlock")
        self.assertEqual(map_ast_classifier_to_m2("PayloadOperator"), "HumanOperator")
        self.assertEqual(map_ast_classifier_to_m2("PilotConsole"), "OperatorConsole")
        self.assertEqual(map_ast_classifier_to_m2("AltimeterWidget"), "Widget")
        self.assertEqual(map_ast_classifier_to_m2("FlightDisplayUI"), "LogicalUI")
        self.assertEqual(map_ast_classifier_to_m2("CANBusPort"), "PortDefinition")
        self.assertEqual(map_ast_classifier_to_m2("OperationalModeState"), "StateDefinition")
        self.assertEqual(map_ast_classifier_to_m2("meta_telemetry_stream"), "meta_telemetry_stream")

    def test_validate_metamodel_purity_catches_violations(self):
        """Verify validate_metamodel_purity detects M1 domain instance dicts and typing leaks."""
        bad_code = """
m1_entities = {"sensor": "Altimeter", "actuator": "ElevatorServo"}
node_spec = {"type": "FlightController", "id": "FC-101"}
sample_uav_specs = {"mass": 15.0}
"""
        violations = validate_metamodel_purity(bad_code, filename="test_bad.py")
        self.assertTrue(len(violations) > 0, "Expected violations for M1 code")
        has_typing_violation = any("domain-metamodel-typing-violation" in str(v) for v in violations)
        self.assertTrue(has_typing_violation, f"Expected domain-metamodel-typing-violation in {violations}")

    def test_validate_metamodel_purity_accepts_clean_m2(self):
        """Verify validate_metamodel_purity passes for valid M2 metamodel constructs."""
        clean_code = """
element = {"type": "Component", "name": "CoreController"}
port_def = {"entity_type": "PortDefinition", "direction": "in"}
meta_node = {"metamodel_type": "meta_custom_binding", "id": "M-1"}
actor_def = {"type": "HumanOperator", "action": "Authorize"}
interlock = {"type": "SafetyInterlock", "threshold": 0.05}
widget = {"type": "Widget", "name": "AltitudeGauge"}
lui = {"type": "LogicalUI", "name": "PrimaryDisplay"}
"""
        violations = validate_metamodel_purity(clean_code, filename="test_clean.py")
        self.assertEqual(violations, [], f"Expected zero violations for clean M2 code, got {violations}")


    def test_validate_metamodel_purity_catches_static_parameter_dict_violations(self):
        """Verify validate_metamodel_purity catches static parameter dicts and functions returning static dicts."""
        bad_static_code = """
STATIC_SPECS = {"mass": 10.0}
GROUND_TRUTH_PARAMS = {"thrust": 45.0}

def extract_ground_truth():
    return {"speed": 120}

class GroundTruthTable:
    PARAMS = {"frequency": 400}
"""
        violations = validate_metamodel_purity(bad_static_code, filename="test_static.py")
        self.assertTrue(len(violations) >= 3, f"Expected multiple violations for static params: {violations}")
        has_static_violation = any("domain-static-param-violation" in str(v) for v in violations)
        self.assertTrue(has_static_violation, f"Expected domain-static-param-violation in {violations}")

    def test_map_ast_classifier_fallback_and_non_string(self):
        """Verify fallback behavior for unknown classifiers and non-string arguments."""
        self.assertEqual(map_ast_classifier_to_m2(None), "PartDefinition")  # type: ignore
        self.assertEqual(map_ast_classifier_to_m2("UnknownCustomBlock"), "PartDefinition")
        self.assertEqual(map_ast_classifier_to_m2(""), "PartDefinition")

    def test_validator_repo_execution(self):
        """Verify MetamodelTransformationEngine.validate behaves correctly with WorkspaceRepository."""
        import tempfile
        import os
        from parity_auditor.core.workspace import WorkspaceRepository

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create upstream marker
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)

            scripts_dir = os.path.join(tmpdir, "scripts")
            os.makedirs(scripts_dir, exist_ok=True)

            test_file = os.path.join(scripts_dir, "test_clean.py")
            with open(test_file, "w", encoding="utf-8") as f:
                f.write('node = {"type": "Component", "name": "Test"}\n')

            repo = WorkspaceRepository(tmpdir)
            validator = MetamodelTransformationEngine()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
