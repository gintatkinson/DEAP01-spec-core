"""
Unit tests for CONOPS and Mission Intent JSON Schema specification contracts.
Addresses Issues #113 and #114.
"""

import json
import os
import re
import sys
import unittest
from typing import Any, Dict, List, Tuple

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

CONOPS_SCHEMA_PATH = os.path.join(REPO_ROOT, ".pipeline", "schemas", "conops_specification_schema.json")
MISSION_INTENT_SCHEMA_PATH = os.path.join(REPO_ROOT, ".pipeline", "schemas", "mission_intent_specification_schema.json")


def _validate_json_schema_instance(schema: Dict[str, Any], instance: Any, path: str = "") -> List[str]:
    """
    Lightweight recursive validator supporting Draft 2020-12 core constraints:
    type, required, properties, items, minItems, enum, additionalProperties.
    Returns list of error messages (empty list if valid).
    """
    errors: List[str] = []

    # Type check
    schema_type = schema.get("type")
    if schema_type:
        type_map = {
            "object": dict,
            "array": list,
            "string": str,
            "number": (int, float),
            "integer": int,
            "boolean": bool,
            "null": type(None),
        }
        if isinstance(schema_type, list):
            valid_types = tuple(type_map[t] for t in schema_type if t in type_map)
            if not isinstance(instance, valid_types):
                errors.append(f"{path}: expected type {schema_type}, got {type(instance).__name__}")
                return errors
        elif schema_type in type_map:
            expected_cls = type_map[schema_type]
            # Handle float vs int edge cases (in JSON schema, int is a number)
            if schema_type == "number" and not isinstance(instance, (int, float)):
                errors.append(f"{path}: expected number, got {type(instance).__name__}")
                return errors
            elif schema_type == "integer" and not (isinstance(instance, int) and not isinstance(instance, bool)):
                errors.append(f"{path}: expected integer, got {type(instance).__name__}")
                return errors
            elif schema_type not in ("number", "integer") and not isinstance(instance, expected_cls):
                errors.append(f"{path}: expected {schema_type}, got {type(instance).__name__}")
                return errors

    # Enum check
    if "enum" in schema:
        if instance not in schema["enum"]:
            errors.append(f"{path}: value {instance!r} not in enum {schema['enum']!r}")

    # Object validation
    if isinstance(instance, dict):
        required_fields = schema.get("required", [])
        for req in required_fields:
            if req not in instance:
                errors.append(f"{path}: missing required property '{req}'")

        props = schema.get("properties", {})
        additional_allowed = schema.get("additionalProperties", True)

        for key, val in instance.items():
            child_path = f"{path}.{key}" if path else key
            if key in props:
                errors.extend(_validate_json_schema_instance(props[key], val, child_path))
            elif additional_allowed is False:
                errors.append(f"{path}: unexpected property '{key}' not allowed by schema")
            elif isinstance(additional_allowed, dict):
                errors.extend(_validate_json_schema_instance(additional_allowed, val, child_path))

    # Array validation
    if isinstance(instance, list):
        min_items = schema.get("minItems")
        if min_items is not None and len(instance) < min_items:
            errors.append(f"{path}: array has {len(instance)} items, minimum is {min_items}")

        max_items = schema.get("maxItems")
        if max_items is not None and len(instance) > max_items:
            errors.append(f"{path}: array has {len(instance)} items, maximum is {max_items}")

        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for idx, item in enumerate(instance):
                child_path = f"{path}[{idx}]"
                errors.extend(_validate_json_schema_instance(item_schema, item, child_path))

    return errors


def _get_valid_sample_conops_payload() -> Dict[str, Any]:
    return {
        "metadata": {
            "system_identifier": "AutoCPS-01",
            "version": "1.0.0",
            "date": "2026-09-02",
            "title": "Concept of Operations for Autonomous Cyber-Physical System",
            "author": "Systems Engineering Lead",
            "classification": "UNCLASSIFIED",
        },
        "operational_context": {
            "scope": "Tactical wide-area surveillance and monitoring",
            "operational_domain": "UAF::OperationalDomain::CivilSecurityAndMonitoring",
            "operational_boundaries": "Designated test corridor bounded by lateral geofence",
            "stakeholder_roster": "Mission Commander, Flight Safety Officer, Remote Operator",
        },
        "deficiencies": {
            "current_operational_baseline": "Predecessor manual radio-controlled operations",
            "operational_deficiencies": "High operator workload, lack of autonomous failsafe containment",
        },
        "proposed_capabilities": {
            "mission_drivers": "Autonomous flight path execution and continuous telemetry",
            "value_proposition": "10x reduction in manual operator fatigue and standardized safety gates",
            "trade_off_analysis": "Redundant digital C2 link vs payload weight budget",
        },
        "user_classes": [
            {
                "id": "UC-01",
                "title": "Range Safety Officer",
                "player_or_operator": "Operator",
                "interfacing_stakeholder": "Air Traffic Control",
                "characteristics": "Experienced in airspace safety, contingency termination protocols",
                "training_type": "Level-3 Range Safety Certification",
                "constraint_source": "FAA Part 107 / EASA Specific Category Regulation",
            }
        ],
        "airspace_sora": {
            "h_max_m": 120.0,
            "theta_impact_deg": 45.0,
            "v_wind_max_mps": 15.0,
            "g_accel_mps2": 9.80665,
            "d_glide_max_m": 50.0,
            "r_grb_meters": 200.0,
            "v_terminal_mps": 25.0,
            "e_impact_joules": 1562.5,
            "ground_risk_class": "GRC-3",
        },
        "uaf_activities": [
            {
                "id": "OA-01",
                "name": "SystemInitialization",
                "description": "Executes power-on Built-In-Tests and sensor calibration",
                "allocation_tag": "/// OperationalAllocation: [OA-01]",
            }
        ],
        "optx_exchanges": [
            {
                "id": "OpTx-01",
                "source_node": "PrimarySensorSubsystem",
                "destination_node": "ControllerLogicSubsystem",
                "information_element": "PrimarySensorState",
                "throughput": "100 Hz",
                "max_latency": "5 ms",
                "criticality": "High (DAL-A)",
            }
        ],
        "environmental_envelopes": {
            "ambient_temperature": "-20 degC to +50 degC",
            "environmental_ingress": "IP67 sealed enclosure",
            "rf_environment": "GNSS-denied resilient with optical odometry fallback",
            "physical_spatial_constraints": "Clearance envelope 2.5m x 2.5m x 1.0m",
        },
        "scenarios": [
            {
                "id": "SCN-01",
                "title": "Nominal Surveillance Flight Thread",
                "steps": [
                    {
                        "step_num": 1,
                        "elapsed_time": "T+00:00:00",
                        "actor": "Flight Controller",
                        "action": "Complete pre-flight BIT and engage autonomous launch sequence",
                        "expected_result": "All subsystem BIT status words return NOMINAL (0x00)",
                        "exit_criterion": "Vehicle airborne and stabilized at transit altitude",
                    }
                ],
            }
        ],
        "maintenance": {
            "o_level": "Organizational: Pre-flight visual inspection, battery hot-swap, BIT verification",
            "i_level": "Intermediate: Actuator servo calibration, sensor recalibration, modular LRU swap",
            "d_level": "Depot: Airframe structural overhaul, composite NDI inspection, flight computer recertification",
        },
        "emergency_matrix": [
            {
                "id": "EMG-01",
                "trigger_event": "Lost C2 Link",
                "detection_mechanism": "Heartbeat timeout > 5.0 s",
                "primary_failsafe_state": "Contingency_LostLinkReturn",
                "max_response_time": "0.50 s",
                "authority_role": "Autonomous / Range Safety Officer Override",
            },
            {
                "id": "EMG-02",
                "trigger_event": "GNSS Navigation Loss",
                "detection_mechanism": "FOM > 3.0 or RAIM Alert",
                "primary_failsafe_state": "Contingency_DeadReckoning",
                "max_response_time": "0.10 s",
                "authority_role": "Autonomous / Operator Monitor",
            },
            {
                "id": "EMG-03",
                "trigger_event": "Propulsion Power Failure",
                "detection_mechanism": "RPM drop / Over-current trip",
                "primary_failsafe_state": "Contingency_EmergencyGlide",
                "max_response_time": "0.05 s",
                "authority_role": "Autonomous Containment",
            },
            {
                "id": "EMG-04",
                "trigger_event": "Critical Sensor Fault",
                "detection_mechanism": "Cross-channel disparity > 3-sigma",
                "primary_failsafe_state": "Degraded_SensorFailsafe",
                "max_response_time": "0.05 s",
                "authority_role": "Autonomous / Operator Monitor",
            },
            {
                "id": "EMG-05",
                "trigger_event": "Geofence Breach Alert",
                "detection_mechanism": "Boundary proximity margin < 50 m",
                "primary_failsafe_state": "Contingency_GeofenceContainment",
                "max_response_time": "0.20 s",
                "authority_role": "Autonomous / Range Safety Officer Override",
            },
            {
                "id": "EMG-06",
                "trigger_event": "Structural Actuation Anomaly",
                "detection_mechanism": "Vibration harmonic threshold exceeded",
                "primary_failsafe_state": "Contingency_PrecautionaryLand",
                "max_response_time": "0.50 s",
                "authority_role": "Autonomous / Remote Pilot Monitor",
            },
            {
                "id": "EMG-07",
                "trigger_event": "Flight Termination Command",
                "detection_mechanism": "Encrypted emergency abort packet received",
                "primary_failsafe_state": "Emergency_FlightTermination",
                "max_response_time": "0.02 s",
                "authority_role": "Range Safety Officer",
            },
        ],
    }


def _get_valid_sample_mission_intent_payload() -> Dict[str, Any]:
    return {
        "commanders_intent": {
            "operational_purpose": "Execute autonomous perimeter surveillance and situational awareness gathering",
            "key_tasks": "Secure takeoff, transit surveillance corridor, stream sensor telemetry, return to base",
            "end_state": "All survey waypoints verified, zero geofence breaches, safe recovery with >20% reserve energy",
        },
        "metl_tasks": [
            {
                "id": "MET-01",
                "task_name": "PreFlightSystemCheckout",
                "condition": "Pre-launch power on at staging point",
                "standard_metric": "100% PBIT pass in < 30 s",
                "verification_method": "Automated BIT Log Verification",
                "allocation_tag": "/// OperationalAllocation: [MET-01]",
            }
        ],
        "incose_moe_mop": [
            {
                "id": "MoE-01",
                "parameter_name": "Mission Area Coverage Ratio",
                "katex_formula": "A_{\\mathrm{covered}} / A_{\\mathrm{total}}",
                "threshold_value": "0.90",
                "objective_value": "0.99",
                "measurement_unit": "Dimensionless",
            }
        ],
        "threat_matrix": [
            {
                "id": "THR-01",
                "domain": "Electronic Warfare",
                "vector": "GNSS Spoofing / Jamming",
                "technical_description": "Loss of GPS carrier lock or pseudo-range disparity > 50 m",
                "severity_category": "High",
                "detection_mechanism": "RAIM Alert and IMU cross-verification",
                "autonomous_mitigation_rule": "Revert to optical dead-reckoning and IMU integration",
                "public_clause_citation": "STANAG 4586 Annex B §3.2.1",
            }
        ],
        "pace_c2_plan": [
            {
                "tier": "Primary",
                "frequency_band": "5.8 GHz ISM",
                "data_rate": "10.0 Mbps",
                "timeout": "2.0 s",
                "failover_hysteresis": "0.5 s",
            },
            {
                "tier": "Alternate",
                "frequency_band": "Band 28 / 700 MHz LTE",
                "data_rate": "2.0 Mbps",
                "timeout": "3.0 s",
                "failover_hysteresis": "1.0 s",
            },
            {
                "tier": "Contingency",
                "frequency_band": "915 MHz ISM FHSS",
                "data_rate": "115.2 kbps",
                "timeout": "5.0 s",
                "failover_hysteresis": "2.0 s",
            },
            {
                "tier": "Emergency",
                "frequency_band": "1.6 GHz L-Band Satellite",
                "data_rate": "2.4 kbps",
                "timeout": "10.0 s",
                "failover_hysteresis": "5.0 s",
            },
        ],
        "roe_interlocks": [
            {
                "id": "ROE-01",
                "rule_statement": "System shall not execute autonomous descent below 30 m without positive radar clearance",
                "interlock_condition": "altitude_agl < 30.0 and not radar_clearance -> hold_altitude",
            }
        ],
        "airspace_geozones": {
            "primary_boundary_perimeter": "Outer polygon bounding perimeter with 50 m warning buffer",
            "dynamic_exclusion_zones": "Populated area buffer circles (R = 300 m) marked NO-FLY",
            "separation_minima": "Maintain 150 m vertical and 500 m horizontal separation from non-cooperative targets",
        },
        "go_no_go_matrix": [
            {
                "id": "GNG-01",
                "phase": "Pre-Launch",
                "parameter": "Battery State of Charge",
                "threshold_condition": "SoC >= 95.0%",
                "sensor_or_mechanism": "Smart Battery BMS",
                "action": "Abort Launch if SoC < 95%",
            }
        ],
        "bingo_energy_math": {
            "e_capacity_joules": 500000.0,
            "e_return_joules": 150000.0,
            "e_divert_joules": 60000.0,
            "e_reserve_joules": 100000.0,
            "e_contingency_joules": 40000.0,
            "e_bingo_threshold_joules": 350000.0,
            "statutory_reserve_ratio": 0.20,
        },
        "allocation_tags": [
            "/// OperationalAllocation: [MET-01]"
        ],
    }


class TestSpecificationSchemas(unittest.TestCase):
    """
    Test suite verifying that CONOPS and Mission Intent JSON Schema specification
    contracts comply with Draft 2020-12 and enforce all structural invariants (Issues #113, #114).
    """

    def test_schema_files_exist(self):
        """Verify that both schema files exist in .pipeline/schemas/."""
        self.assertTrue(
            os.path.isfile(CONOPS_SCHEMA_PATH),
            f"Missing CONOPS schema at {CONOPS_SCHEMA_PATH}",
        )
        self.assertTrue(
            os.path.isfile(MISSION_INTENT_SCHEMA_PATH),
            f"Missing Mission Intent schema at {MISSION_INTENT_SCHEMA_PATH}",
        )

    def test_conops_schema_is_valid_draft_2020_12_json(self):
        """Verify CONOPS schema parses as valid JSON and declares Draft 2020-12 meta-schema."""
        with open(CONOPS_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        self.assertIn("$schema", schema)
        self.assertIn("2020-12", schema["$schema"])
        self.assertEqual(schema.get("type"), "object")

        required_sections = [
            "metadata",
            "operational_context",
            "deficiencies",
            "proposed_capabilities",
            "user_classes",
            "airspace_sora",
            "uaf_activities",
            "optx_exchanges",
            "environmental_envelopes",
            "scenarios",
            "maintenance",
            "emergency_matrix",
        ]
        for sec in required_sections:
            self.assertIn(sec, schema.get("required", []), f"Missing required section '{sec}' in CONOPS schema")
            self.assertIn(sec, schema.get("properties", {}), f"Missing property definition '{sec}' in CONOPS schema")

    def test_conops_schema_array_cardinalities(self):
        """Verify CONOPS schema enforces minimum cardinalities on open array definitions."""
        with open(CONOPS_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        props = schema["properties"]

        # user_classes: minItems >= 1
        self.assertEqual(props["user_classes"].get("type"), "array")
        self.assertGreaterEqual(props["user_classes"].get("minItems", 0), 1)
        uc_items = props["user_classes"].get("items", {})
        uc_req = uc_items.get("required", [])
        for fld in ["id", "title", "player_or_operator", "interfacing_stakeholder", "characteristics", "training_type", "constraint_source"]:
            self.assertIn(fld, uc_req, f"user_classes item missing required field '{fld}'")

        # scenarios: minItems >= 1
        self.assertEqual(props["scenarios"].get("type"), "array")
        self.assertGreaterEqual(props["scenarios"].get("minItems", 0), 1)
        scn_items = props["scenarios"].get("items", {})
        scn_req = scn_items.get("required", [])
        for fld in ["id", "title", "steps"]:
            self.assertIn(fld, scn_req, f"scenarios item missing required field '{fld}'")
        steps_prop = scn_items.get("properties", {}).get("steps", {})
        self.assertEqual(steps_prop.get("type"), "array")
        self.assertGreaterEqual(steps_prop.get("minItems", 0), 1)
        step_req = steps_prop.get("items", {}).get("required", [])
        for fld in ["step_num", "elapsed_time", "actor", "action", "expected_result", "exit_criterion"]:
            self.assertIn(fld, step_req, f"scenario step missing required field '{fld}'")

        # optx_exchanges: minItems >= 1
        self.assertEqual(props["optx_exchanges"].get("type"), "array")
        self.assertGreaterEqual(props["optx_exchanges"].get("minItems", 0), 1)
        optx_items = props["optx_exchanges"].get("items", {})
        optx_req = optx_items.get("required", [])
        for fld in ["id", "source_node", "destination_node", "information_element", "throughput", "max_latency"]:
            self.assertIn(fld, optx_req, f"optx_exchanges item missing required field '{fld}'")

        # emergency_matrix: minItems >= 7
        self.assertEqual(props["emergency_matrix"].get("type"), "array")
        self.assertGreaterEqual(props["emergency_matrix"].get("minItems", 0), 7)
        emg_items = props["emergency_matrix"].get("items", {})
        emg_req = emg_items.get("required", [])
        for fld in ["id", "trigger_event", "detection_mechanism", "primary_failsafe_state", "max_response_time", "authority_role"]:
            self.assertIn(fld, emg_req, f"emergency_matrix item missing required field '{fld}'")

    def test_mission_intent_schema_is_valid_draft_2020_12_json(self):
        """Verify Mission Intent schema parses as valid JSON and declares Draft 2020-12 meta-schema."""
        with open(MISSION_INTENT_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        self.assertIn("$schema", schema)
        self.assertIn("2020-12", schema["$schema"])
        self.assertEqual(schema.get("type"), "object")

        required_sections = [
            "commanders_intent",
            "metl_tasks",
            "incose_moe_mop",
            "threat_matrix",
            "pace_c2_plan",
            "roe_interlocks",
            "airspace_geozones",
            "go_no_go_matrix",
            "bingo_energy_math",
            "allocation_tags",
        ]
        for sec in required_sections:
            self.assertIn(sec, schema.get("required", []), f"Missing required section '{sec}' in Mission Intent schema")
            self.assertIn(sec, schema.get("properties", {}), f"Missing property definition '{sec}' in Mission Intent schema")

    def test_mission_intent_schema_array_cardinalities(self):
        """Verify Mission Intent schema enforces open array definitions and minimum cardinalities."""
        with open(MISSION_INTENT_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        props = schema["properties"]

        # metl_tasks: minItems >= 1
        self.assertEqual(props["metl_tasks"].get("type"), "array")
        self.assertGreaterEqual(props["metl_tasks"].get("minItems", 0), 1)
        metl_req = props["metl_tasks"].get("items", {}).get("required", [])
        for fld in ["id", "task_name", "condition", "standard_metric", "verification_method", "allocation_tag"]:
            self.assertIn(fld, metl_req, f"metl_tasks item missing required field '{fld}'")

        # threat_matrix: minItems >= 1
        self.assertEqual(props["threat_matrix"].get("type"), "array")
        self.assertGreaterEqual(props["threat_matrix"].get("minItems", 0), 1)
        thr_req = props["threat_matrix"].get("items", {}).get("required", [])
        for fld in ["id", "domain", "vector", "technical_description", "severity_category", "detection_mechanism", "autonomous_mitigation_rule", "public_clause_citation"]:
            self.assertIn(fld, thr_req, f"threat_matrix item missing required field '{fld}'")

        # incose_moe_mop: minItems >= 1
        self.assertEqual(props["incose_moe_mop"].get("type"), "array")
        self.assertGreaterEqual(props["incose_moe_mop"].get("minItems", 0), 1)
        moe_req = props["incose_moe_mop"].get("items", {}).get("required", [])
        for fld in ["id", "parameter_name", "katex_formula", "threshold_value", "objective_value", "measurement_unit"]:
            self.assertIn(fld, moe_req, f"incose_moe_mop item missing required field '{fld}'")

        # pace_c2_plan: minItems >= 4
        self.assertEqual(props["pace_c2_plan"].get("type"), "array")
        self.assertGreaterEqual(props["pace_c2_plan"].get("minItems", 0), 4)
        pace_req = props["pace_c2_plan"].get("items", {}).get("required", [])
        for fld in ["tier", "frequency_band", "data_rate", "timeout", "failover_hysteresis"]:
            self.assertIn(fld, pace_req, f"pace_c2_plan item missing required field '{fld}'")

    def test_sample_conops_payload_validates_against_schema(self):
        """Verify that a valid CONOPS JSON payload passes schema validation with zero errors."""
        with open(CONOPS_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        payload = _get_valid_sample_conops_payload()
        errors = _validate_json_schema_instance(schema, payload)
        self.assertEqual(errors, [], f"Valid CONOPS payload failed schema validation: {errors}")

    def test_sample_mission_intent_payload_validates_against_schema(self):
        """Verify that a valid Mission Intent JSON payload passes schema validation with zero errors."""
        with open(MISSION_INTENT_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        payload = _get_valid_sample_mission_intent_payload()
        errors = _validate_json_schema_instance(schema, payload)
        self.assertEqual(errors, [], f"Valid Mission Intent payload failed schema validation: {errors}")

    def test_conops_schema_rejects_missing_required_section(self):
        """Verify that CONOPS schema rejects payloads missing mandatory sections."""
        with open(CONOPS_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        payload = _get_valid_sample_conops_payload()
        del payload["emergency_matrix"]

        errors = _validate_json_schema_instance(schema, payload)
        self.assertTrue(any("missing required property 'emergency_matrix'" in err for err in errors), errors)

    def test_conops_schema_rejects_insufficient_emergency_rows(self):
        """Verify that CONOPS schema rejects emergency_matrix with fewer than 7 rows."""
        with open(CONOPS_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        payload = _get_valid_sample_conops_payload()
        payload["emergency_matrix"] = payload["emergency_matrix"][:6]  # 6 rows instead of 7

        errors = _validate_json_schema_instance(schema, payload)
        self.assertTrue(any("minimum is 7" in err for err in errors), errors)

    def test_mission_intent_schema_rejects_insufficient_pace_tiers(self):
        """Verify that Mission Intent schema rejects pace_c2_plan with fewer than 4 tiers."""
        with open(MISSION_INTENT_SCHEMA_PATH, "r", encoding="utf-8") as f:
            schema = json.load(f)

        payload = _get_valid_sample_mission_intent_payload()
        payload["pace_c2_plan"] = payload["pace_c2_plan"][:3]  # 3 tiers instead of 4

        errors = _validate_json_schema_instance(schema, payload)
        self.assertTrue(any("minimum is 4" in err for err in errors), errors)


if __name__ == "__main__":
    unittest.main()
