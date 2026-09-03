"""
Unit and integration tests for assemble_conops.py deterministic assembly engine
and SysML AST Parameter Binding Engine.
Addresses Issues #113, #114, and Fixes #143.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from typing import Dict, List

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

from scripts.assemble_conops import (
    CANONICAL_CONOPS_UNITS,
    CANONICAL_MISSION_INTENT_UNITS,
    DEFAULT_CONOPS_PARAMS,
    RAW_TOKEN_FINDER,
    SysMLParameterBindingEngine,
    assemble_conops,
    assemble_document,
    bind_parameters,
    extract_headings,
    generate_table_of_contents,
    validate_unit_integrity,
    verify_markdown_links,
)


def _create_sample_conops_units(units_dir: str, with_placeholders: bool = False, with_empty_unit: bool = False):
    """Populates mock conops unit files in units_dir."""
    os.makedirs(units_dir, exist_ok=True)

    token_val = "{{SYSTEM_IDENTIFIER}}" if with_placeholders else "AutonomousCyberPhysicalSystem"

    units = {
        "01_METADATA_AND_OVERVIEW.md": f"""# Concept of Operations (ConOps): {token_val}

## 1. Scope & System Identification
- **System Identifier:** `{token_val}`
- **Operational Domain:** `UAF::OperationalDomain::AutonomousMonitoring`
- **Operational Boundaries:** Defined polygon within designated operational range.
- **Stakeholder Roster:** Range Safety Officer, Remote Operator in Command, Payload Specialist.
""",
        "02_DEFICIENCIES_AND_MOTIVATION.md": """## 2. Current Situation & Deficiency Analysis (Predecessors)
- **Current Operational Baseline:** Manual line-of-sight tele-operation with analog telemetry.
- **Operational Deficiencies:** Lack of autonomous failsafe containment, range limited by line-of-sight.
""",
        "03_PROPOSED_CAPABILITIES.md": """## 3. Operational Justification & Priority Matrix (Trade-Offs)
- **Mission Drivers & Value Proposition:** High-tempo continuous perimeter monitoring with automated geo-fencing.
- **Trade-Off Analysis:** Dedicated satellite backup link vs battery payload mass budget.
""",
        "04_USER_CLASSES_AND_STAKEHOLDERS.md": """## 4. Operational Modes & Lifecycle Stages
Formal operational lifecycle stages across $\\Phi_{\\mathrm{lifecycle}}$:
- **Phase_Startup:** Built-In-Test self-check and navigation calibration.
- **Phase_NominalExecution:** Automated waypoint tracking and payload monitoring.
- **Phase_DegradedMode:** Sensor redundancy failsafe mode.
- **Phase_ContingencyFailsafe:** Autonomous return-to-base and controlled containment.
- **Phase_SecureShutdown:** Post-mission payload encryption and shutdown.
- **Phase_MaintenanceMode:** Diagnostic telemetry analysis and component swap.
""",
        "05_AIRSPACE_AND_SORA_RISK.md": """## 5. 4D Operational Volume & SORA Ground Risk Buffer Mathematics
$$
\\begin{aligned}
V_{\\mathrm{4D}} &= V_{\\mathrm{SpatialGeometry}} \\cup V_{\\mathrm{ContingencyVolume}} \\cup V_{\\mathrm{GRB}} \\\\
R_{\\mathrm{GRB}} &= h_{\\mathrm{max}} \\cdot \\tan(\\theta_{\\mathrm{impact}}) + v_{\\mathrm{wind,max}} \\cdot \\sqrt{\\frac{2 h_{\\mathrm{max}}}{g}} + d_{\\mathrm{glide,max}}
\\end{aligned}
$$

| Parameter | Symbol | Value | Units | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max Altitude / Ceiling | h_max | 120.0 | m | Maximum operating ceiling above reference surface |
| Impact Angle | theta_impact | 45.0 | deg | Worst-case operational trajectory impact angle |
| Max Wind Speed | v_wind_max | 15.0 | m/s | Maximum operational wind speed limit |
| Gravitational Accel | g | 9.80665 | m/s^2 | Standard gravitational acceleration constant |
| Maximum Glide Distance | d_glide_max | 50.0 | m | Maximum unpowered lateral displacement margin |
| Ground Risk Buffer Radius | R_GRB | 200.0 | m | Declared ground risk buffer containment radius |
| Terminal Velocity | v_terminal | 25.0 | m/s | Estimated unpowered descent terminal velocity |
| Impact Kinetic Energy | E_impact | 1562.5 | J | Kinetic energy at operational boundary impact |
""",
        "06_UAF_OPERATIONAL_ACTIVITIES.md": """## 6. OMG UAF Operational Activity Taxonomy
| Activity ID | Activity Name | Description | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- |
| OA-01 | PreFlightBIT | Executes power-on Built-In-Tests and sensor calibration | `/// OperationalAllocation: [OA-01]` |
| OA-02 | ExecuteTrajectoryTracking | Performs closed-loop waypoint guidance along corridor | `/// OperationalAllocation: [OA-02]` |
""",
        "07_OPTX_EXCHANGES.md": """## 7. Operational Information Exchange (Op-Tx) Matrix
| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| OpTx-01 | PrimarySensorSubsystem | ControllerLogicSubsystem | PrimarySensorState | 100 Hz | 5 ms | High (DAL-A) |
| OpTx-02 | ControllerLogicSubsystem | ActuatorSubsystem | ActuatorDemandValue | 200 Hz | 2.5 ms | High (DAL-A) |
""",
        "08_ENVIRONMENTAL_MIL_STD_810H.md": """## 8. Operational Environments & Constraints
- **Ambient Temperature:** -20 degC to +50 degC
- **Environmental Ingress:** IP67 environmental sealing
- **Electromagnetic / RF Environment:** Resilient against intentional GNSS denial
- **Physical Spatial Constraints:** Launch footprint < 3m x 3m
""",
        "09_SCENARIOS_AND_TIMELINES.md": """## 9. Multi-Threaded Operational Scenarios
- **Scenario 1 (Nominal Execution):** Autonomous pre-flight BIT, launch, corridor survey, and precision recovery.
- **Scenario 2 (Degraded Mode & Mitigation):** Primary GPS loss triggers optical odometry navigation fallback.
- **Scenario 3 (Contingency Recovery):** Total C2 lost-link triggers return-to-base rally point sequence.
""",
        "10_MAINTENANCE_AND_GSE_SUPPORT.md": """## 10. Maintenance & Sustainment Concepts (O/I/D Maintenance)
- **O-Level (Organizational):** Pre-flight visual check, battery hot-swap, BIT verification.
- **I-Level (Intermediate):** Actuator servo calibration, sensor recalibration, modular LRU swap.
- **D-Level (Depot):** Chassis structural overhaul, NDI inspection, computer recertification.
""",
        "11_IMPACTS_AND_TRADE_STUDIES.md": """## 11. Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title / Baseline | Applicable Clauses |
| :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps & §6.4.3 OpsCon |
| OMG UAF v1.2 / v2.0 | OMG | Unified Architecture Framework | Operational Domain (Op-*) |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB) |
""",
        "12_EMERGENCY_DECISION_MATRIX.md": """## 12. 7-Row Emergency Decision & Contingency Matrix
| Trigger ID | Contingency Trigger | Detection Mechanism | Automated Containment Action | Failsafe State | Max Response Time | HITL Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EMG-01` | Lost C2 Link | Heartbeat loss > 5.0 s | Execute autonomous lost-link loiter / return | `Contingency_LostLinkReturn` | 0.50 s | Monitor / Override |
| `EMG-02` | GNSS Navigation Loss | FOM > 3.0 or RAIM Alert | Switch to dead-reckoning / optical odometry | `Contingency_DeadReckoning` | 0.10 s | Monitor / Override |
| `EMG-03` | Propulsion Failure | RPM drop / Over-current | Execute glide to nearest secondary divert site | `Contingency_EmergencyGlide` | 0.05 s | Informed |
| `EMG-04` | Critical Sensor Fault | Cross-channel disparity | Revert to simplex failsafe sensor mode | `Degraded_SensorFailsafe` | 0.05 s | Monitor |
| `EMG-05` | Geofence Breach Alert | Boundary proximity < 50 m | Execute emergency turnaround maneuver | `Contingency_GeofenceContainment` | 0.20 s | Monitor / Override |
| `EMG-06` | Structural Anomaly | Vibration threshold exceeded | Throttle reduction and immediate landing | `Contingency_PrecautionaryLand` | 0.50 s | Monitor / Override |
| `EMG-07` | Flight Termination Cmd | Encrypted abort signal | Deploy containment / instant motor cutoff | `Emergency_FlightTermination` | 0.02 s | Initiator |
""",
    }

    if with_empty_unit:
        units["13_empty_stray.md"] = "   \n\n  "

    for filename, content in units.items():
        with open(os.path.join(units_dir, filename), "w", encoding="utf-8") as f:
            f.write(content)


def _create_sample_mission_intent_units(units_dir: str, with_placeholders: bool = False):
    """Populates mock mission intent unit files in units_dir."""
    os.makedirs(units_dir, exist_ok=True)

    token_val = "{{MISSION_SYSTEM_NAME}}" if with_placeholders else "AutonomousCyberPhysicalSystem"

    units = {
        "01_COMMANDERS_INTENT.md": f"""# Tactical Mission Intent & Execution Plan: {token_val}

## 1. Commander's Intent & Operational Objectives
- **Operational Purpose:** Execute autonomous wide-area perimeter surveillance and monitoring.
- **Key Tasks:** Secure startup, transit surveillance corridor, stream sensor telemetry, return to base.
- **End State:** All survey waypoints verified, zero geofence breaches, safe recovery with >20% reserve energy.
""",
        "02_MISSION_ESSENTIAL_TASK_LIST.md": """## 2. Mission Essential Task List (METL)
| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `MET-01` | PreFlightSystemCheckout | Pre-launch power on | 100% PBIT pass in < 30 s | Automated BIT Log | `/// OperationalAllocation: [MET-01]` |
| `MET-02` | AutonomousIngressTransit | En-route nominal corridor | Cross-track error < 2.0 m | Flight Log Review | `/// OperationalAllocation: [MET-02]` |
""",
        "03_INCOSE_MOE_MOP_MATH.md": """## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Mission Area Coverage Ratio | A_covered / A_total | 0.90 | 0.99 | Dimensionless |
| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | 5.0 | 1.0 | m |
| MoP-02 | MoP | Telemetry Latency Bound | tau_transport | 50.0 | 10.0 | ms |
""",
        "04_MULTI_DOMAIN_THREAT_MATRIX.md": """## 4. Multi-Domain Operational Threat & Contested Environment Matrix
| Threat ID | Threat Vector | Description | Severity | Autonomous Mitigation Rule |
| :--- | :--- | :--- | :--- | :--- |
| THR-01 | GNSS Spoofing / Jamming | Loss of carrier lock or pseudo-range jump | High | Revert to optical dead-reckoning and IMU integration |
""",
        "05_PACE_C2_PLAN.md": """## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Priority / Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary** | Point-to-Point COFDM | 5.8 GHz ISM | 10.0 Mbps | 2.0 s | Video & High-rate Telemetry |
| **Alternate** | Cellular LTE / 5G VPN | Band 28 / 700 MHz | 2.0 Mbps | 3.0 s | Encrypted Cloud Relay |
| **Contingency** | 900 MHz FHSS Radio | 915 MHz ISM | 115.2 kbps | 5.0 s | Essential C2 Commands Only |
| **Emergency** | Satellite Iridium SBD | 1.6 GHz L-Band | 2.4 kbps | 10.0 s | Emergency Flight Termination & Geo-Beacon |
""",
        "06_ROE_SAFETY_INTERLOCKS.md": """## 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks
- **ROE-01:** System shall not execute autonomous descent below 30 m without positive ground radar clearance.
""",
        "07_AIRSPACE_GEOZONES.md": """## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones
- **Primary Boundary Perimeter:** Outer polygon bounding perimeter with 50 m warning buffer.
- **Dynamic Exclusion Zones:** Populated area buffer circles (R = 300 m) marked NO-FLY.
- **Separation Minima:** Maintain 150 m vertical and 500 m horizontal separation from non-cooperative targets.
""",
        "08_GO_NO_GO_MATRIX.md": """## 8. Go/No-Go Decision Matrix
| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| GNG-01 | Pre-Launch | Battery State of Charge | >= 95.0% | Smart Battery BMS | Abort Launch if < 95% |
""",
        "09_BINGO_ENERGY_MATH.md": """## 9. Bingo Energy Mathematics & Secondary Divert Protocols
$$
\\begin{aligned}
E_{\\mathrm{bingo}}(t) &= E_{\\mathrm{return}}(\\mathbf{p}(t), \\mathbf{p}_{\\mathrm{dest}}) + E_{\\mathrm{divert}}(\\mathbf{p}_{\\mathrm{dest}}, \\mathbf{p}_{\\mathrm{alt}}) + E_{\\mathrm{reserve}} + E_{\\mathrm{contingency}} \\\\
E_{\\mathrm{reserve}} &\\ge 0.20 \\cdot E_{\\mathrm{capacity}}
\\end{aligned}
$$

| Energy Parameter | Symbol | Value | Units | Constraint Rule |
| :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | 500000.0 | J | Total nominal energy storage capacity |
| Return Transit Energy | E_return | 150000.0 | J | Energy required for primary return trajectory |
| Secondary Divert Energy | E_divert | 60000.0 | J | Energy required to divert to secondary recovery site |
| Mandatory Statutory Reserve | E_reserve | 100000.0 | J | Statutory reserve threshold (E_reserve >= 0.20 * E_capacity) |
| Contingency Buffer | E_contingency | 40000.0 | J | Dynamic operational contingency energy reserve |
| Total Bingo Threshold | E_bingo | 350000.0 | J | Critical return threshold condition |
""",
        "10_OPERATIONAL_ALLOCATION_TAGS.md": """## 10. Gate 24 MissionTask Traceability Tags (Allocation Tags)
- `/// OperationalAllocation: [MET-01]`
- `/// OperationalAllocation: [MET-02]`
""",
    }

    for filename, content in units.items():
        with open(os.path.join(units_dir, filename), "w", encoding="utf-8") as f:
            f.write(content)


class TestAssembleConops(unittest.TestCase):
    """
    Test suite for scripts/assemble_conops.py deterministic assembly engine (Issues #113, #114, #143).
    """

    def test_extract_headings_and_generate_toc(self):
        """Verify heading extraction and table-of-contents generation."""
        sample_md = """# My Main Document

## 1. Scope & System Identification
Some text here.

### 1.1 Stakeholders
Stakeholder list.

## 2. Standards
Standards list.
"""
        headings = extract_headings(sample_md)
        self.assertEqual(len(headings), 4)
        self.assertEqual(headings[0], (1, "My Main Document", "my-main-document"))
        self.assertEqual(headings[1], (2, "1. Scope & System Identification", "1-scope--system-identification"))
        self.assertEqual(headings[2], (3, "1.1 Stakeholders", "11-stakeholders"))
        self.assertEqual(headings[3], (2, "2. Standards", "2-standards"))

        toc = generate_table_of_contents(headings, max_depth=2)
        self.assertIn("[1. Scope & System Identification](#1-scope--system-identification)", toc)
        self.assertIn("[2. Standards](#2-standards)", toc)

    def test_validate_unit_integrity_detects_empty_file(self):
        """Verify that empty unit files trigger unit integrity errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            empty_file = os.path.join(tmpdir, "01_empty.md")
            with open(empty_file, "w", encoding="utf-8") as f:
                f.write("   \n\n\t  ")

            valid, errors = validate_unit_integrity([empty_file])
            self.assertFalse(valid)
            self.assertTrue(any("is empty" in err for err in errors))

    def test_validate_unit_integrity_detects_unresolved_placeholder_tokens(self):
        """Verify that unresolved template placeholder tokens trigger unit integrity errors when no param engine is active."""
        with tempfile.TemporaryDirectory() as tmpdir:
            unit_file = os.path.join(tmpdir, "01_unit.md")
            with open(unit_file, "w", encoding="utf-8") as f:
                f.write("# Concept of Operations: {{SYSTEM_IDENTIFIER}}\n\nSome text {{OA_01_DESCRIPTION}}.")

            valid, errors = validate_unit_integrity([unit_file])
            self.assertFalse(valid)
            self.assertTrue(any("unresolved placeholder token" in err for err in errors))
            self.assertTrue(any("{{SYSTEM_IDENTIFIER}}" in err for err in errors))

    def test_validate_unit_integrity_passes_clean_units(self):
        """Verify that valid, populated unit files pass integrity checks."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_sample_conops_units(tmpdir, with_placeholders=False)
            unit_files = [os.path.join(tmpdir, f) for f in sorted(os.listdir(tmpdir)) if f.endswith(".md")]

            valid, errors = validate_unit_integrity(unit_files)
            self.assertTrue(valid, f"Unit integrity failed on clean units: {errors}")
            self.assertEqual(errors, [])

    def test_assemble_document_creates_complete_markdown_with_toc(self):
        """Verify assemble_document compiles ordered units into a unified document with header and TOC."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_units_dir = os.path.join(tmpdir, "units", "conops")
            _create_sample_conops_units(conops_units_dir, with_placeholders=False)

            doc_text, errors = assemble_document(
                units_dir=conops_units_dir,
                doc_title="Concept of Operations (ConOps)",
                doc_version="1.0.0",
                doc_date="2026-09-02",
            )
            self.assertEqual(errors, [])
            self.assertIn("# Concept of Operations (ConOps)", doc_text)
            self.assertIn("## Table of Contents", doc_text)
            self.assertIn("## 1. Scope & System Identification", doc_text)
            self.assertIn("## 12. 7-Row Emergency Decision & Contingency Matrix", doc_text)
            self.assertIn("EMG-01", doc_text)
            self.assertIn("EMG-07", doc_text)

            # Check for 0 broken relative internal links
            link_errors = verify_markdown_links(doc_text)
            self.assertEqual(link_errors, [], f"Broken anchor links found: {link_errors}")

    def test_assemble_conops_full_pipeline(self):
        """Verify full assemble_conops workflow producing CONOPS.md and MISSION_INTENT.md."""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_dir = os.path.join(tmpdir, "units")
            output_dir = os.path.join(tmpdir, "docs", "conops")

            conops_units_dir = os.path.join(input_dir, "conops")
            mission_units_dir = os.path.join(input_dir, "mission_intent")

            _create_sample_conops_units(conops_units_dir, with_placeholders=False)
            _create_sample_mission_intent_units(mission_units_dir, with_placeholders=False)

            success = assemble_conops(
                input_dir=input_dir,
                output_dir=output_dir,
                verify_only=False,
            )
            self.assertTrue(success)

            conops_path = os.path.join(output_dir, "CONOPS.md")
            mission_path = os.path.join(output_dir, "MISSION_INTENT.md")

            self.assertTrue(os.path.isfile(conops_path), "Missing compiled CONOPS.md")
            self.assertTrue(os.path.isfile(mission_path), "Missing compiled MISSION_INTENT.md")

            with open(conops_path, "r", encoding="utf-8") as f:
                conops_content = f.read()
            with open(mission_path, "r", encoding="utf-8") as f:
                mission_content = f.read()

            self.assertIn("## 1. Scope & System Identification", conops_content)
            self.assertIn("## 12. 7-Row Emergency Decision & Contingency Matrix", conops_content)
            self.assertIn("## 1. Commander's Intent & Operational Objectives", mission_content)
            self.assertIn("## 10. Gate 24 MissionTask Traceability Tags", mission_content)

            # Re-run in verify mode
            verify_success = assemble_conops(
                input_dir=input_dir,
                output_dir=output_dir,
                verify_only=True,
            )
            self.assertTrue(verify_success)

    def test_cli_execution_and_flags(self):
        """Verify assemble_conops.py script CLI interface."""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_dir = os.path.join(tmpdir, "units")
            output_dir = os.path.join(tmpdir, "out")

            conops_units_dir = os.path.join(input_dir, "conops")
            mission_units_dir = os.path.join(input_dir, "mission_intent")

            _create_sample_conops_units(conops_units_dir, with_placeholders=False)
            _create_sample_mission_intent_units(mission_units_dir, with_placeholders=False)

            script_path = os.path.join(REPO_ROOT, "scripts", "assemble_conops.py")

            # Run via subprocess
            res = subprocess.run(
                [sys.executable, script_path, "--input-dir", input_dir, "--output-dir", output_dir],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res.returncode, 0, f"CLI failed with stderr: {res.stderr}\nstdout: {res.stdout}")
            self.assertTrue(os.path.isfile(os.path.join(output_dir, "CONOPS.md")))
            self.assertTrue(os.path.isfile(os.path.join(output_dir, "MISSION_INTENT.md")))

            # Run with --verify flag
            res_verify = subprocess.run(
                [sys.executable, script_path, "--input-dir", input_dir, "--output-dir", output_dir, "--verify"],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res_verify.returncode, 0, f"Verify CLI failed: {res_verify.stderr}\nstdout: {res_verify.stdout}")


class TestSysMLParameterBindingEngine(unittest.TestCase):
    """
    Unit and integration tests for SysMLParameterBindingEngine (Issue #143).
    """

    def test_parameter_engine_initialization_with_dict(self):
        """Verify initialization with direct parameter values."""
        engine = SysMLParameterBindingEngine(
            parameter_values={"SYSTEM_IDENTIFIER": "CustomPlatform", "V_CRUISE_NOMINAL_MPS": "30.0"},
            auto_detect=False,
        )
        self.assertEqual(engine.resolve_token("SYSTEM_IDENTIFIER"), "CustomPlatform")
        self.assertEqual(engine.resolve_token("V_CRUISE_NOMINAL_MPS"), "30.0")

    def test_parameter_engine_nested_dictionary_ingestion(self):
        """Verify ingestion of nested configuration dictionaries."""
        nested = {
            "metadata": {
                "system_name": "AutonomousEdgeNode",
                "version": "2.1.0",
            },
            "parameters": {
                "V_CRUISE_NOMINAL_MPS": "28.5",
                "CEILING_MAX_M": "4500.0",
                "MASS_FRACTION_PAYLOAD_PCT": "12.0",
            },
        }
        engine = SysMLParameterBindingEngine(parameter_values=nested, auto_detect=False)
        self.assertEqual(engine.resolve_token("SYSTEM_IDENTIFIER"), "AutonomousEdgeNode")
        self.assertEqual(engine.resolve_token("V_CRUISE_NOMINAL_MPS"), "28.5")
        self.assertEqual(engine.resolve_token("CEILING_MAX_M"), "4500.0")
        self.assertEqual(engine.resolve_token("MASS_FRACTION_PAYLOAD_PCT"), "12.0")

    def test_parameter_engine_json_file_ingestion(self):
        """Verify parameter ingestion from an external JSON file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            json_path = os.path.join(tmpdir, "custom_params.json")
            data = {
                "system_identifier": "DistributedSensorArray",
                "TOTAL_MTOW_KG": "48.0",
                "INGRESS_PROTECTION_RATING": "IP68",
                "M501_OP_HIGH_TEMP_C": "60.0",
            }
            with open(json_path, "w", encoding="utf-8") as f:
                json.dump(data, f)

            engine = SysMLParameterBindingEngine(config_path=json_path, auto_detect=False)
            self.assertEqual(engine.resolve_token("SYSTEM_IDENTIFIER"), "DistributedSensorArray")
            self.assertEqual(engine.resolve_token("TOTAL_MTOW_KG"), "48.0")
            self.assertEqual(engine.resolve_token("INGRESS_PROTECTION_RATING"), "IP68")
            self.assertEqual(engine.resolve_token("M501_OP_HIGH_TEMP_C"), "60.0")

    def test_parameter_engine_sysml_ast_ingestion(self):
        """Verify SysML v2 textual AST parsing and semantic alias mapping."""
        sysml_code = """
        package AutonomousRoboticVehicle {
            attribute mtow : Real = 45.0;
            attribute v_cruise : Real = 22.0;
            attribute max_altitude : Real = 3500.0;
            attribute temp_min : Real = -25.0;
            attribute temp_max : Real = 60.0;
            assert constraint SpeedLimit { v_max <= 40.0 }
        }
        """
        engine = SysMLParameterBindingEngine(auto_detect=False)
        engine.ingest_sysml_text(sysml_code)

        self.assertEqual(engine.resolve_token("SYSTEM_IDENTIFIER"), "AutonomousRoboticVehicle")
        self.assertEqual(engine.resolve_token("TOTAL_MTOW_KG"), "45.0")
        self.assertEqual(engine.resolve_token("V_CRUISE_NOMINAL_MPS"), "22.0")
        self.assertEqual(engine.resolve_token("CEILING_MAX_M"), "3500.0")
        self.assertEqual(engine.resolve_token("H_MAX_M"), "3500.0")
        self.assertEqual(engine.resolve_token("TEMP_MIN_DEGC"), "-25.0")
        self.assertEqual(engine.resolve_token("TEMP_MAX_DEGC"), "60.0")

    def test_parameter_engine_auto_detection_schema_digest(self):
        """Verify auto-detection of .pipeline/schema-digest.json in workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            pipeline_dir = os.path.join(tmpdir, ".pipeline")
            os.makedirs(pipeline_dir, exist_ok=True)
            digest_path = os.path.join(pipeline_dir, "schema-digest.json")

            digest_content = {
                "sha256": "abc12345",
                "system_name": "PipelineIngestedSystem",
                "parameters": {
                    "TOTAL_MTOW_KG": "65.0",
                    "C2_RANGE_NOMINAL_KM": "75.0",
                },
            }
            with open(digest_path, "w", encoding="utf-8") as f:
                json.dump(digest_content, f)

            engine = SysMLParameterBindingEngine(workspace_dir=tmpdir, auto_detect=True)
            self.assertEqual(engine.resolve_token("SYSTEM_IDENTIFIER"), "PipelineIngestedSystem")
            self.assertEqual(engine.resolve_token("TOTAL_MTOW_KG"), "65.0")
            self.assertEqual(engine.resolve_token("C2_RANGE_NOMINAL_KM"), "75.0")

    def test_substitute_canonical_template_placeholders(self):
        """Verify substitution of all canonical template placeholders with sensible defaults."""
        engine = SysMLParameterBindingEngine(auto_detect=False)

        sample_template = """
# System: {{SYSTEM_IDENTIFIER}}
- Version: {{DOCUMENT_VERSION}}
- Date: {{DOCUMENT_DATE}}
- Security: {{SECURITY_CLASSIFICATION}}
- Target Realization: {{TARGET_SYSTEM_REALIZATION}}
- Authoring Org: {{AUTHORING_ORGANIZATION}}
- Mass Airframe: {{MASS_FRACTION_AIRFRAME_PCT}}% ({{MASS_BUDGET_AIRFRAME_KG}} kg)
- Cruise Velocity: {{V_CRUISE_NOMINAL_MPS}} m/s
- Ceiling: {{CEILING_MAX_M}} m
- Ambient Temp Range: {{AMBIENT_TEMPERATURE_RANGE}}
- SORA Ground Risk Buffer: {{R_GRB_METERS}} m
- Bingo Energy Capacity: {{E_CAPACITY_JOULES}} J
- M-500 Limits: {{M500_OP_LIMIT}}
- M-501 Limits: {{M501_OP_LIMIT}}
- M-514 Vibration: {{M514_OP_LIMIT}}
- Maintenance SLA: {{RAPID_TURNAROUND_SLA_MIN}} min
- Inline Default Weight: {{TLX_WEIGHT_MD:0.25}}
- Inline Default Timeout: {{HANDOFF_TIMEOUT_SEC:5.0}}
"""
        resolved = engine.substitute(sample_template)

        self.assertNotIn("{{", resolved)
        self.assertNotIn("}}", resolved)
        self.assertIn("AutonomousCyberPhysicalSystem", resolved)
        self.assertIn("1.0.0", resolved)
        self.assertIn("UNCLASSIFIED // PUBLIC RELEASE", resolved)
        self.assertIn("30.0%", resolved)
        self.assertIn("15.0 kg", resolved)
        self.assertIn("25.0 m/s", resolved)
        self.assertIn("5000.0 m", resolved)
        self.assertIn("200.0 m", resolved)
        self.assertIn("500000.0 J", resolved)
        self.assertIn("0.25", resolved)
        self.assertIn("5.0", resolved)

    def test_bind_parameters_convenience_helper(self):
        """Verify bind_parameters convenience function."""
        text = "System identifier is {{SYSTEM_IDENTIFIER}} with ceiling {{CEILING_MAX_M}} m."
        res = bind_parameters(text, {"SYSTEM_IDENTIFIER": "CustomAlpha", "CEILING_MAX_M": "6000.0"})
        self.assertEqual(res, "System identifier is CustomAlpha with ceiling 6000.0 m.")

    def test_assemble_document_with_parameter_binding_resolves_placeholders(self):
        """Verify assemble_document substitutes template placeholders during assembly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_units_dir = os.path.join(tmpdir, "conops")
            _create_sample_conops_units(conops_units_dir, with_placeholders=True)

            engine = SysMLParameterBindingEngine(
                parameter_values={"SYSTEM_IDENTIFIER": "AutonomousPatrolSystem"},
                auto_detect=False,
            )

            doc_text, errors = assemble_document(
                units_dir=conops_units_dir,
                doc_title="Concept of Operations (ConOps)",
                params=engine,
            )
            self.assertEqual(errors, [], f"Assemble document had errors: {errors}")
            self.assertIn("Concept of Operations (ConOps): AutonomousPatrolSystem", doc_text)
            self.assertIn("- **System Identifier:** `AutonomousPatrolSystem`", doc_text)
            self.assertNotIn("{{SYSTEM_IDENTIFIER}}", doc_text)

    def test_assemble_conops_with_custom_params_dict(self):
        """Verify assemble_conops full pipeline with custom parameter dictionary."""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_dir = os.path.join(tmpdir, "units")
            output_dir = os.path.join(tmpdir, "out")

            conops_units_dir = os.path.join(input_dir, "conops")
            mission_units_dir = os.path.join(input_dir, "mission_intent")

            _create_sample_conops_units(conops_units_dir, with_placeholders=True)
            _create_sample_mission_intent_units(mission_units_dir, with_placeholders=True)

            custom_params = {
                "SYSTEM_IDENTIFIER": "ModularInspectionSystem",
                "MISSION_SYSTEM_NAME": "ModularInspectionSystem",
                "DOCUMENT_VERSION": "3.0.0",
            }

            success = assemble_conops(
                input_dir=input_dir,
                output_dir=output_dir,
                verify_only=False,
                params=custom_params,
            )
            self.assertTrue(success)

            conops_file = os.path.join(output_dir, "CONOPS.md")
            mission_file = os.path.join(output_dir, "MISSION_INTENT.md")

            with open(conops_file, "r", encoding="utf-8") as f:
                c_text = f.read()
            with open(mission_file, "r", encoding="utf-8") as f:
                m_text = f.read()

            self.assertIn("ModularInspectionSystem", c_text)
            self.assertIn("ModularInspectionSystem", m_text)
            self.assertNotIn("{{SYSTEM_IDENTIFIER}}", c_text)
            self.assertNotIn("{{MISSION_SYSTEM_NAME}}", m_text)

    def test_cli_params_flag(self):
        """Verify CLI execution with --params <path> flag."""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_dir = os.path.join(tmpdir, "units")
            output_dir = os.path.join(tmpdir, "out")
            params_file = os.path.join(tmpdir, "domain_params.json")

            conops_units_dir = os.path.join(input_dir, "conops")
            mission_units_dir = os.path.join(input_dir, "mission_intent")

            _create_sample_conops_units(conops_units_dir, with_placeholders=True)
            _create_sample_mission_intent_units(mission_units_dir, with_placeholders=True)

            with open(params_file, "w", encoding="utf-8") as f:
                json.dump({"SYSTEM_IDENTIFIER": "CLISystemUnderTest", "MISSION_SYSTEM_NAME": "CLISystemUnderTest"}, f)

            script_path = os.path.join(REPO_ROOT, "scripts", "assemble_conops.py")

            res = subprocess.run(
                [
                    sys.executable,
                    script_path,
                    "--input-dir",
                    input_dir,
                    "--output-dir",
                    output_dir,
                    "--params",
                    params_file,
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res.returncode, 0, f"CLI --params failed: {res.stderr}\nstdout: {res.stdout}")

            conops_out = os.path.join(output_dir, "CONOPS.md")
            with open(conops_out, "r", encoding="utf-8") as f:
                c_text = f.read()
            self.assertIn("CLISystemUnderTest", c_text)
            self.assertNotIn("{{SYSTEM_IDENTIFIER}}", c_text)

    def test_canonical_resource_units_assembly_passes_cleanly(self):
        """Verify end-to-end assembly on actual canonical resource units in skills/spec-conops-engineering/resources/units."""
        res_units_dir = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units")
        if not os.path.isdir(res_units_dir):
            self.skipTest(f"Resource units dir not found at {res_units_dir}")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out")
            success = assemble_conops(
                input_dir=res_units_dir,
                output_dir=out_dir,
                verify_only=False,
            )
            self.assertTrue(success, "assemble_conops failed on canonical resource units")

            conops_path = os.path.join(out_dir, "CONOPS.md")
            mission_path = os.path.join(out_dir, "MISSION_INTENT.md")

            self.assertTrue(os.path.isfile(conops_path))
            self.assertTrue(os.path.isfile(mission_path))

            with open(conops_path, "r", encoding="utf-8") as f:
                conops_text = f.read()
            with open(mission_path, "r", encoding="utf-8") as f:
                mission_text = f.read()

            # Verify zero unresolved placeholders
            self.assertEqual(RAW_TOKEN_FINDER.findall(conops_text), [])
            self.assertEqual(RAW_TOKEN_FINDER.findall(mission_text), [])

            # Verify all 12 ConOps sections present
            for sec_num in range(1, 13):
                self.assertIn(f"## {sec_num}.", conops_text)

            # Verify all 10 Mission Intent sections present
            for sec_num in range(1, 11):
                self.assertIn(f"## {sec_num}.", mission_text)

    def test_assemble_conops_skips_and_warns_on_non_canonical_units(self):
        """Verify assemble_document skips non-canonical / deprecated units and only compiles whitelisted units (Fixes #148)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_units_dir = os.path.join(tmpdir, "conops")
            _create_sample_conops_units(conops_units_dir, with_placeholders=False)

            # Inject ghost/deprecated files
            ghost_file_1 = os.path.join(conops_units_dir, "00_deprecated_preface.md")
            ghost_file_2 = os.path.join(conops_units_dir, "13_extraneous_appendix.md")
            ghost_file_3 = os.path.join(conops_units_dir, "deprecated_notes.md")

            with open(ghost_file_1, "w", encoding="utf-8") as f:
                f.write("## 0. Deprecated Preface\nGhost content 1\n")
            with open(ghost_file_2, "w", encoding="utf-8") as f:
                f.write("## 13. Extraneous Appendix\nGhost content 2\n")
            with open(ghost_file_3, "w", encoding="utf-8") as f:
                f.write("## Ghost Notes\nGhost content 3\n")

            doc, errors = assemble_document(
                units_dir=conops_units_dir,
                doc_title="Concept of Operations (ConOps)",
                canonical_whitelist=CANONICAL_CONOPS_UNITS,
            )
            self.assertEqual(errors, [], f"Unexpected errors during assembly: {errors}")
            self.assertNotIn("Ghost content 1", doc)
            self.assertNotIn("Ghost content 2", doc)
            self.assertNotIn("Ghost content 3", doc)
            self.assertNotIn("## 0. Deprecated Preface", doc)
            self.assertNotIn("## 13. Extraneous Appendix", doc)

            # Ensure all 12 canonical units are assembled
            for sec_num in range(1, 13):
                self.assertIn(f"## {sec_num}.", doc)

    def test_assemble_conops_toc_contains_all_12_and_10_sections_with_valid_anchors(self):
        """Verify assembled CONOPS and MISSION_INTENT generate a complete TOC with all sections and valid anchor links (Fixes #148)."""
        res_units_dir = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units")
        if not os.path.isdir(res_units_dir):
            self.skipTest(f"Resource units dir not found at {res_units_dir}")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out")
            success = assemble_conops(
                input_dir=res_units_dir,
                output_dir=out_dir,
                verify_only=False,
            )
            self.assertTrue(success)

            conops_path = os.path.join(out_dir, "CONOPS.md")
            mission_path = os.path.join(out_dir, "MISSION_INTENT.md")

            with open(conops_path, "r", encoding="utf-8") as f:
                conops_text = f.read()
            with open(mission_path, "r", encoding="utf-8") as f:
                mission_text = f.read()

            # Verify ConOps Table of Contents has all 12 sections
            self.assertIn("## Table of Contents", conops_text)
            toc_match_conops = re.search(r'## Table of Contents[\s\S]*?(?=## 1\.)', conops_text)
            self.assertIsNotNone(toc_match_conops)
            toc_conops = toc_match_conops.group(0)
            for i in range(1, 13):
                self.assertRegex(toc_conops, rf'\[(?:Section\s+)?{i}\.')

            # Verify Mission Intent Table of Contents has all 10 sections
            self.assertIn("## Table of Contents", mission_text)
            toc_match_mission = re.search(r'## Table of Contents[\s\S]*?(?=## 1\.)', mission_text)
            self.assertIsNotNone(toc_match_mission)
            toc_mission = toc_match_mission.group(0)
            for i in range(1, 11):
                self.assertRegex(toc_mission, rf'\[(?:Section\s+)?{i}\.')

    def test_default_conops_params_bindings(self):
        """Verify DEFAULT_CONOPS_PARAMS bindings and resolution in SysMLParameterBindingEngine."""
        self.assertEqual(DEFAULT_CONOPS_PARAMS["MAX_JUNCTION_TEMPERATURE_DELTA_C"], "25.0")
        self.assertEqual(DEFAULT_CONOPS_PARAMS["BATTERY_CHARGE_C_RATE"], "2.0C")
        self.assertEqual(DEFAULT_CONOPS_PARAMS["BATTERY_CHARGE_TIME_HOURS"], "1.5")
        self.assertEqual(DEFAULT_CONOPS_PARAMS["SUPPORT_EQUIPMENT_BATTERY_HOURS"], "8.0")
        self.assertEqual(DEFAULT_CONOPS_PARAMS["OPERATIONAL_AVAILABILITY_THRESHOLD"], "0.95")
        self.assertEqual(DEFAULT_CONOPS_PARAMS["OPERATIONAL_AVAILABILITY_OBJECTIVE"], "0.99")

        engine = SysMLParameterBindingEngine(auto_detect=False)
        for key, expected_val in DEFAULT_CONOPS_PARAMS.items():
            self.assertEqual(engine.resolve_token(key), expected_val)

    def test_operational_intent_explicit_ingestion(self):
        """Verify SysMLParameterBindingEngine ingests explicit OPERATIONAL_PURPOSE and PRIMARY_OPERATIONAL_MISSION (Fixes #157)."""
        explicit_params = {
            "OPERATIONAL_PURPOSE": "Custom operational purpose for test vehicle.",
            "PRIMARY_OPERATIONAL_MISSION": "Custom primary operational mission for test vehicle.",
        }
        engine = SysMLParameterBindingEngine(parameter_values=explicit_params, auto_detect=False)
        self.assertEqual(engine.resolve_token("OPERATIONAL_PURPOSE"), "Custom operational purpose for test vehicle.")
        self.assertEqual(engine.resolve_token("PRIMARY_OPERATIONAL_MISSION"), "Custom primary operational mission for test vehicle.")

        template = "- Purpose: {{OPERATIONAL_PURPOSE}}\n- Mission: {{PRIMARY_OPERATIONAL_MISSION}}"
        substituted = engine.substitute(template)
        self.assertIn("- Purpose: Custom operational purpose for test vehicle.", substituted)
        self.assertIn("- Mission: Custom primary operational mission for test vehicle.", substituted)

    def test_operational_intent_derivation_from_ast_entities(self):
        """Verify SysMLParameterBindingEngine derives domain-grounded purpose & mission from AST entities (Fixes #157)."""
        domain_params = {
            "SYSTEM_NAME": "AeroScan X1",
            "PLATFORM_TYPE": "Autonomous Aerial Vehicle",
            "OPERATIONAL_DOMAIN": "Maritime Surveillance",
            "PRIMARY_COMMS": "COFDM Mesh Link",
            "REGULATORY_CLASS": "EASA Specific SAIL III",
        }
        engine = SysMLParameterBindingEngine(parameter_values=domain_params, auto_detect=False)

        expected_purpose = (
            "The primary operational purpose of AeroScan X1 (Autonomous Aerial Vehicle) is to execute deterministic, "
            "autonomous operational tasks, real-time multi-modal state monitoring, and safety-critical boundary containment "
            "within designated Maritime Surveillance environments, operating under verified EASA Specific SAIL III governance "
            "and resilient COFDM Mesh Link communications."
        )
        expected_mission = (
            "The AeroScan X1 is engineered to execute high-assurance Maritime Surveillance missions, autonomous closed-loop control, "
            "telemetry processing, and deterministic contingency containment in compliance with EASA Specific SAIL III requirements."
        )

        self.assertEqual(engine.resolve_token("OPERATIONAL_PURPOSE"), expected_purpose)
        self.assertEqual(engine.resolve_token("PRIMARY_OPERATIONAL_MISSION"), expected_mission)

    def test_operational_intent_default_synthesis(self):
        """Verify SysMLParameterBindingEngine synthesizes deterministic fallback defaults when parameters are omitted (Fixes #157)."""
        engine = SysMLParameterBindingEngine(auto_detect=False)

        expected_purpose = (
            "The primary operational purpose of Autonomous Cyber-Physical System (Cyber-Physical System) is to execute deterministic, "
            "autonomous operational tasks, real-time multi-modal state monitoring, and safety-critical boundary containment "
            "within designated Autonomous Operations environments, operating under verified High-Assurance Safety Baseline governance "
            "and resilient Multi-Tier C2 Telemetry Link communications."
        )
        expected_mission = (
            "The Autonomous Cyber-Physical System is engineered to execute high-assurance Autonomous Operations missions, autonomous closed-loop control, "
            "telemetry processing, and deterministic contingency containment in compliance with High-Assurance Safety Baseline requirements."
        )

        self.assertEqual(engine.resolve_token("OPERATIONAL_PURPOSE"), expected_purpose)
        self.assertEqual(engine.resolve_token("PRIMARY_OPERATIONAL_MISSION"), expected_mission)

    def test_core_mission_capabilities_explicit_ingestion(self):
        """Verify SysMLParameterBindingEngine ingests explicit CORE_MISSION_CAPABILITIES and individual CORE_CAPABILITY_1..4 (Fixes #158)."""
        explicit_params = {
            "CORE_MISSION_CAPABILITIES": "  1. Surgical arm trajectory tracking.\n  2. Endoscopic camera sensor fusion.\n  3. Low-latency haptic telemetry.\n  4. Patient boundary containment interlock.",
        }
        engine = SysMLParameterBindingEngine(parameter_values=explicit_params, auto_detect=False)
        self.assertEqual(engine.resolve_token("CORE_MISSION_CAPABILITIES"), explicit_params["CORE_MISSION_CAPABILITIES"])

        individual_caps = {
            "CORE_CAPABILITY_1": "Custom Capability 1",
            "CORE_CAPABILITY_2": "Custom Capability 2",
            "CORE_CAPABILITY_3": "Custom Capability 3",
            "CORE_CAPABILITY_4": "Custom Capability 4",
        }
        engine2 = SysMLParameterBindingEngine(parameter_values=individual_caps, auto_detect=False)
        expected = "  1. Custom Capability 1\n  2. Custom Capability 2\n  3. Custom Capability 3\n  4. Custom Capability 4"
        self.assertEqual(engine2.resolve_token("CORE_MISSION_CAPABILITIES"), expected)

        list_caps = {
            "CORE_MISSION_CAPABILITIES": [
                "List Capability 1",
                "List Capability 2",
            ]
        }
        engine3 = SysMLParameterBindingEngine(parameter_values=list_caps, auto_detect=False)
        expected_list = "  1. List Capability 1\n  2. List Capability 2"
        self.assertEqual(engine3.resolve_token("CORE_MISSION_CAPABILITIES"), expected_list)

    def test_core_mission_capabilities_derivation_from_ast_entities(self):
        """Verify SysMLParameterBindingEngine derives domain-grounded capabilities from AST entities (Fixes #158)."""
        domain_params = {
            "PLATFORM_TYPE": "Subsea Autonomous Underwater Vehicle",
            "OPERATIONAL_DOMAIN": "Deep Ocean Bathymetry",
            "PRIMARY_COMMS": "Acoustic & Optical Modem",
            "REGULATORY_CLASS": "DNV-GL Underwater Assurance",
        }
        engine = SysMLParameterBindingEngine(parameter_values=domain_params, auto_detect=False)
        expected_c1 = "Autonomous closed-loop state trajectory tracking, corridor execution, and operational boundary holding for Subsea Autonomous Underwater Vehicle in Deep Ocean Bathymetry."
        expected_c2 = "Multi-modal sensor data fusion combining redundant state estimation sensors, environmental perception units, and reference state observers."
        expected_c3 = "Real-time high-throughput telemetry streaming and deterministic command processing over Acoustic & Optical Modem."
        expected_c4 = "Deterministic failsafe state machine ensuring autonomous containment within response threshold limits in accordance with DNV-GL Underwater Assurance."
        expected_full = f"  1. {expected_c1}\n  2. {expected_c2}\n  3. {expected_c3}\n  4. {expected_c4}"

        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_1"), expected_c1)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_2"), expected_c2)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_3"), expected_c3)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_4"), expected_c4)
        self.assertEqual(engine.resolve_token("CORE_MISSION_CAPABILITIES"), expected_full)

    def test_core_mission_capabilities_default_synthesis(self):
        """Verify SysMLParameterBindingEngine synthesizes deterministic fallback default capabilities when parameters are omitted (Fixes #158)."""
        engine = SysMLParameterBindingEngine(auto_detect=False)
        expected_c1 = "Autonomous closed-loop state trajectory tracking, corridor execution, and operational boundary holding for Cyber-Physical System in Autonomous Operations."
        expected_c2 = "Multi-modal sensor data fusion combining redundant state estimation sensors, environmental perception units, and reference state observers."
        expected_c3 = "Real-time high-throughput telemetry streaming and deterministic command processing over Multi-Tier C2 Telemetry Link."
        expected_c4 = "Deterministic failsafe state machine ensuring autonomous containment within response threshold limits in accordance with High-Assurance Safety Baseline."
        expected_full = f"  1. {expected_c1}\n  2. {expected_c2}\n  3. {expected_c3}\n  4. {expected_c4}"

        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_1"), expected_c1)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_2"), expected_c2)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_3"), expected_c3)
        self.assertEqual(engine.resolve_token("CORE_CAPABILITY_4"), expected_c4)
        self.assertEqual(engine.resolve_token("CORE_MISSION_CAPABILITIES"), expected_full)


if __name__ == "__main__":
    unittest.main()
