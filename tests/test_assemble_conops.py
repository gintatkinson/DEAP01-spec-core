"""
Unit and integration tests for assemble_conops.py deterministic assembly engine.
Addresses Issues #113 and #114.
"""

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
    assemble_conops,
    assemble_document,
    validate_unit_integrity,
    generate_table_of_contents,
    verify_markdown_links,
    extract_headings,
)


def _create_sample_conops_units(units_dir: str, with_placeholders: bool = False, with_empty_unit: bool = False):
    """Populates mock conops unit files in units_dir."""
    os.makedirs(units_dir, exist_ok=True)

    token_val = "{{SYSTEM_IDENTIFIER}}" if with_placeholders else "AutonomousSurveillanceUAS"

    units = {
        "01_scope.md": f"""# Concept of Operations (ConOps): {token_val}

## 1. Scope & System Identification
- **System Identifier:** `{token_val}`
- **Operational Domain:** `UAF::OperationalDomain::CivilSecurityAndMonitoring`
- **Operational Boundaries:** Defined polygon within designated flight test range.
- **Stakeholder Roster:** Range Safety Officer, Remote Pilot in Command, Payload Specialist.
""",
        "02_standards.md": """## 2. Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title / Baseline | Applicable Clauses |
| :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps & §6.4.3 OpsCon |
| OMG UAF v1.2 / v2.0 | OMG | Unified Architecture Framework | Operational Domain (Op-*) |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB) |
""",
        "03_deficiencies.md": """## 3. Current Situation & Deficiency Analysis (Predecessors)
- **Current Operational Baseline:** Manual line-of-sight visual piloting with analog telemetry.
- **Operational Deficiencies:** Lack of autonomous failsafe containment, range limited by line-of-sight.
""",
        "04_capabilities.md": """## 4. Operational Justification & Priority Matrix (Trade-Offs)
- **Mission Drivers & Value Proposition:** High-tempo continuous perimeter monitoring with automated geo-fencing.
- **Trade-Off Analysis:** Dedicated satellite backup link vs battery payload mass budget.
""",
        "05_lifecycle.md": """## 5. Operational Modes & Lifecycle Stages
Formal operational lifecycle stages across $\\Phi_{\\mathrm{lifecycle}}$:
- **Phase_Startup:** Built-In-Test self-check and navigation calibration.
- **Phase_NominalExecution:** Automated waypoint tracking and payload monitoring.
- **Phase_DegradedMode:** Sensor redundancy failsafe mode.
- **Phase_ContingencyFailsafe:** Autonomous return-to-base and controlled containment.
- **Phase_SecureShutdown:** Post-flight payload encryption and shutdown.
- **Phase_MaintenanceMode:** Diagnostic telemetry analysis and component swap.
""",
        "06_sora.md": """## 6. 4D Operational Volume & SORA Ground Risk Buffer Mathematics
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
        "07_uaf_activities.md": """## 7. OMG UAF Operational Activity Taxonomy
| Activity ID | Activity Name | Description | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- |
| OA-01 | PreFlightBIT | Executes power-on Built-In-Tests and sensor calibration | `/// OperationalAllocation: [OA-01]` |
| OA-02 | ExecuteTrajectoryTracking | Performs closed-loop waypoint guidance along corridor | `/// OperationalAllocation: [OA-02]` |
""",
        "08_optx_matrix.md": """## 8. Operational Information Exchange (Op-Tx) Matrix
| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| OpTx-01 | PrimarySensorSubsystem | ControllerLogicSubsystem | PrimarySensorState | 100 Hz | 5 ms | High (DAL-A) |
| OpTx-02 | ControllerLogicSubsystem | ActuatorSubsystem | ActuatorDemandValue | 200 Hz | 2.5 ms | High (DAL-A) |
""",
        "09_environments.md": """## 9. Operational Environments & Constraints
- **Ambient Temperature:** -20 degC to +50 degC
- **Environmental Ingress:** IP67 environmental sealing
- **Electromagnetic / RF Environment:** Resilient against intentional GNSS denial
- **Physical Spatial Constraints:** Launch footprint < 3m x 3m
""",
        "10_scenarios.md": """## 10. Multi-Threaded Operational Scenarios
- **Scenario 1 (Nominal Execution):** Autonomous pre-flight BIT, launch, corridor survey, and precision recovery.
- **Scenario 2 (Degraded Mode & Mitigation):** Primary GPS loss triggers optical odometry navigation fallback.
- **Scenario 3 (Contingency Recovery):** Total C2 lost-link triggers return-to-base rally point sequence.
""",
        "11_maintenance.md": """## 11. Maintenance & Sustainment Concepts (O/I/D Maintenance)
- **O-Level (Organizational):** Pre-flight visual check, battery hot-swap, BIT verification.
- **I-Level (Intermediate):** Actuator servo calibration, sensor recalibration, modular LRU swap.
- **D-Level (Depot):** Airframe structural overhaul, composite NDI inspection, flight computer recertification.
""",
        "12_emergency_matrix.md": """## 12. 7-Row Emergency Decision & Contingency Matrix
| Trigger ID | Contingency Trigger | Detection Mechanism | Automated Containment Action | Failsafe State | Max Response Time | HITL Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EMG-01` | Lost C2 Link | Heartbeat loss > 5.0 s | Execute autonomous lost-link loiter / return | `Contingency_LostLinkReturn` | 0.50 s | Monitor / Override |
| `EMG-02` | GNSS Navigation Loss | FOM > 3.0 or RAIM Alert | Switch to dead-reckoning / optical odometry | `Contingency_DeadReckoning` | 0.10 s | Monitor / Override |
| `EMG-03` | Propulsion Failure | RPM drop / Over-current | Execute glide to nearest secondary divert site | `Contingency_EmergencyGlide` | 0.05 s | Informed |
| `EMG-04` | Critical Sensor Fault | Cross-channel disparity | Revert to simplex failsafe sensor mode | `Degraded_SensorFailsafe` | 0.05 s | Monitor |
| `EMG-05` | Geofence Breach Alert | Boundary proximity < 50 m | Execute emergency turnaround maneuver | `Contingency_GeofenceContainment` | 0.20 s | Monitor / Override |
| `EMG-06` | Structural Anomaly | Vibration threshold exceeded | Throttle reduction and immediate landing | `Contingency_PrecautionaryLand` | 0.50 s | Monitor / Override |
| `EMG-07` | Flight Termination Cmd | Encrypted abort signal | Deploy parachute / instant motor cutoff | `Emergency_FlightTermination` | 0.02 s | Initiator |
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

    token_val = "{{MISSION_SYSTEM_NAME}}" if with_placeholders else "AutonomousSurveillanceUAS"

    units = {
        "01_intent.md": f"""# Tactical Mission Intent & Execution Plan: {token_val}

## 1. Commander's Intent & Operational Objectives
- **Operational Purpose:** Execute autonomous wide-area perimeter surveillance and monitoring.
- **Key Tasks:** Secure takeoff, transit surveillance corridor, stream sensor telemetry, return to base.
- **End State:** All survey waypoints verified, zero geofence breaches, safe recovery with >20% reserve energy.
""",
        "02_metl.md": """## 2. Mission Essential Task List (METL)
| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `MET-01` | PreFlightSystemCheckout | Pre-launch power on | 100% PBIT pass in < 30 s | Automated BIT Log | `/// OperationalAllocation: [MET-01]` |
| `MET-02` | AutonomousIngressTransit | En-route nominal corridor | Cross-track error < 2.0 m | Flight Log Review | `/// OperationalAllocation: [MET-02]` |
""",
        "03_moe_mop.md": """## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Mission Area Coverage Ratio | A_covered / A_total | 0.90 | 0.99 | Dimensionless |
| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | 5.0 | 1.0 | m |
| MoP-02 | MoP | Telemetry Latency Bound | tau_transport | 50.0 | 10.0 | ms |
""",
        "04_threats.md": """## 4. Threat & Electronic Warfare (EW) / Cyber Environment Matrix
| Threat ID | Threat Vector | Description | Severity | Autonomous Mitigation Rule |
| :--- | :--- | :--- | :--- | :--- |
| THR-01 | GNSS Spoofing / Jamming | Loss of carrier lock or pseudo-range jump | High | Revert to optical dead-reckoning and IMU integration |
""",
        "05_pace.md": """## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Priority / Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary** | Point-to-Point COFDM | 5.8 GHz ISM | 10.0 Mbps | 2.0 s | Video & High-rate Telemetry |
| **Alternate** | Cellular LTE / 5G VPN | Band 28 / 700 MHz | 2.0 Mbps | 3.0 s | Encrypted Cloud Relay |
| **Contingency** | 900 MHz FHSS Radio | 915 MHz ISM | 115.2 kbps | 5.0 s | Essential C2 Commands Only |
| **Emergency** | Satellite Iridium SBD | 1.6 GHz L-Band | 2.4 kbps | 10.0 s | Emergency Flight Termination & Geo-Beacon |
""",
        "06_roe.md": """## 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks
- **ROE-01:** System shall not execute autonomous descent below 30 m without positive ground radar clearance.
""",
        "07_airspace.md": """## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones
- **Primary Boundary Perimeter:** Outer polygon bounding perimeter with 50 m warning buffer.
- **Dynamic Exclusion Zones:** Populated area buffer circles (R = 300 m) marked NO-FLY.
- **Separation Minima:** Maintain 150 m vertical and 500 m horizontal separation from non-cooperative targets.
""",
        "08_gng.md": """## 8. Go/No-Go Decision Matrix
| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| GNG-01 | Pre-Launch | Battery State of Charge | >= 95.0% | Smart Battery BMS | Abort Launch if < 95% |
""",
        "09_bingo.md": """## 9. Bingo Energy Mathematics & Secondary Divert Protocols
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
        "10_tags.md": """## 10. Gate 24 MissionTask Traceability Tags (Allocation Tags)
- `/// OperationalAllocation: [MET-01]`
- `/// OperationalAllocation: [MET-02]`
""",
    }

    for filename, content in units.items():
        with open(os.path.join(units_dir, filename), "w", encoding="utf-8") as f:
            f.write(content)


class TestAssembleConops(unittest.TestCase):
    """
    Test suite for scripts/assemble_conops.py deterministic assembly engine (Issues #113, #114).
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
        """Verify that unresolved template placeholder tokens trigger unit integrity errors."""
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


if __name__ == "__main__":
    unittest.main()
