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
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from parity_auditor.validators.conops_completeness_validator import (
    ConopsCompletenessValidator,
    MissionIntentCompletenessValidator,
    ConopsDocumentModel,
    MissionIntentDocumentModel,
    OperationalVolume4D,
    SoraRiskBuffer,
    BingoEnergyModel,
    EmergencyDecisionRow,
    METLTaskEntry,
    MoEMoPMetricEntry,
    PaceC2LinkPlan,
    calculate_sora_grb_radius,
    calculate_bingo_energy_reserve_ratio,
)


def _get_valid_conops_content() -> str:
    return """| Attribute | Value |
| :--- | :--- |
| **Title** | Concept of Operations (ConOps): Autonomous Cyber-Physical System |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |

# Concept of Operations (ConOps): Autonomous Cyber-Physical System

## 1. Scope & System Identification
- **System Identifier:** `AutonomousSystemArchetype`
- **Operational Domain:** `UAF::OperationalDomain::CivilSecurityAndMonitoring`
- **System Boundaries:** Bounded air/ground operational zone within designated test range.
- **Stakeholder Roster:** Range Safety Officer, System Operator, Payload Specialist.

## 2. Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title / Baseline | Applicable Clauses |
| :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps & §6.4.3 OpsCon |
| OMG UAF v1.2 / v2.0 | OMG | Unified Architecture Framework | Operational Domain (Op-*) |
| NATO STANAG 4586 | NATO | Standard Interfaces of UAV Control System (UCS) | Interoperability Profiles |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB) |
| RTCA DO-178C / DO-254 | RTCA | Software and Airborne Electronic Hardware Considerations | Safety Assurance |

## 3. Current Situation & Deficiency Analysis (Predecessors)
- **Current Operational Baseline:** Predecessor manual radio-controlled operations.
- **Operational Deficiencies:** Limited endurance, manual sensor telemetry parsing, single point of link failure.

## 4. Operational Justification & Priority Matrix (Trade-Offs)
- **Mission Drivers:** Autonomous flight path execution and real-time situational telemetry.
- **Trade-Off Analysis:** Redundant digital C2 link vs payload weight penalty.

## 5. Operational Modes & Lifecycle Stages
Formal operational lifecycle stages across $\\Phi_{\\mathrm{lifecycle}}$:
- **Phase_Startup:** System power-on Built-In-Test (BIT) and calibration.
- **Phase_NominalExecution:** Active mission execution and waypoint tracking.
- **Phase_DegradedMode:** Subsystem failure with automated failsafe mitigation.
- **Phase_ContingencyFailsafe:** Autonomous return-to-base and safety containment.
- **Phase_SecureShutdown:** Power down and secure data offloading.
- **Phase_MaintenanceMode:** Diagnostic telemetry offload and component maintenance.

## 6. 4D Operational Volume & SORA Ground Risk Buffer Mathematics
4D spatial-temporal envelope:
$$
\\begin{aligned}
V_{\\mathrm{4D}} &= V_{\\mathrm{FlightGeometry}} \\cup V_{\\mathrm{ContingencyVolume}} \\cup V_{\\mathrm{GRB}} \\\\
R_{\\mathrm{GRB}} &= h_{\\mathrm{max}} \\cdot \\tan(\\theta_{\\mathrm{impact}}) + v_{\\mathrm{wind,max}} \\cdot \\sqrt{\\frac{2 h_{\\mathrm{max}}}{g}}
\\end{aligned}
$$

| Parameter | Symbol | Value | Units | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max Altitude AGL | h_max | 120.0 | m | Maximum operating ceiling above ground |
| Impact Angle | theta_impact | 45.0 | deg | SORA 1:1 rule worst-case impact vector |
| Max Wind Speed | v_wind_max | 15.0 | m/s | Maximum operational wind limit |
| Gravitational Accel | g | 9.80665 | m/s^2 | Standard gravitational acceleration |
| Ground Risk Buffer Radius | R_GRB | 200.0 | m | Declared ground risk buffer radius |
| Terminal Velocity | v_terminal | 25.0 | m/s | Estimated unpowered descent terminal velocity |
| Impact Kinetic Energy | E_impact | 1562.5 | J | Kinetic energy at ground impact (m=5.0 kg) |

## 7. OMG UAF Operational Activity Taxonomy
| Activity ID | Activity Name | Description | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- |
| OA-01 | SystemInitialization | Executes power-on Built-In-Tests | `/// OperationalAllocation: [OA-01]` |
| OA-02 | ExecuteTrajectoryTracking | Performs closed-loop waypoint guidance | `/// OperationalAllocation: [OA-02]` |
| OA-03 | HealthMonitoring | Performs continuous cross-channel sanity checks | `/// OperationalAllocation: [OA-03]` |

## 8. Operational Information Exchange (Op-Tx) Matrix
| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| OpTx-01 | PrimarySensorSubsystem | ControllerLogicSubsystem | PrimarySensorState | 100 Hz | 5 ms | High (DAL-A) |
| OpTx-02 | ControllerLogicSubsystem | ActuatorSubsystem | ActuatorDemandValue | 200 Hz | 2.5 ms | High (DAL-A) |

## 9. Operational Environments & Constraints
- **Ambient Temperature:** $-20^\\circ\\text{C}$ to $+50^\\circ\\text{C}$.
- **Precipitation Limit:** $5\\text{ mm/hr}$ continuous rain.
- **RF Environment:** GNSS-degraded operations with fallback to dead reckoning.

## 10. Multi-Threaded Operational Scenarios
- **Scenario 1 (Nominal):** Normal takeoff, waypoint traversal, mission payload capture, and precision landing.
- **Scenario 2 (Degraded Sensor):** Primary IMU glitch triggers fallback to secondary optical sensor.

## 11. Maintenance & Sustainment Concepts (O/I/D Maintenance)
- **O-Level (Organizational):** Pre-flight pre-arm checklist, battery hot-swap, visual propeller inspection.
- **I-Level (Intermediate):** Actuator servo calibration, sensor recalibration, LRU modular replacement.
- **D-Level (Depot):** Airframe structural overhaul, composite NDI testing, flight controller recertification.

## 12. 7-Row Emergency Decision & Contingency Matrix (MBD Simulink Integration & Traceability)
| Trigger ID | Contingency Trigger | Detection Mechanism | Automated Containment Action | Failsafe State | Max Response Time | HITL Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EMG-01` | Lost C2 Link | Heartbeat loss > 5.0 s | Execute autonomous lost-link loiter / return | `Contingency_LostLinkReturn` | 0.50 s | Monitor / Override |
| `EMG-02` | GNSS Navigation Loss | FOM > 3.0 or RAIM Alert | Switch to dead-reckoning / optical odometry | `Contingency_DeadReckoning` | 0.10 s | Monitor / Override |
| `EMG-03` | Propulsion Failure | RPM drop / Over-current | Execute glide to nearest secondary divert site | `Contingency_EmergencyGlide` | 0.05 s | Informed |
| `EMG-04` | Critical Sensor Fault | Cross-channel disparity | Revert to simplex failsafe sensor mode | `Degraded_SensorFailsafe` | 0.05 s | Monitor |
| `EMG-05` | Geofence Breach Alert | Boundary proximity < 50 m | Execute emergency turnaround maneuver | `Contingency_GeofenceContainment` | 0.20 s | Monitor / Override |
| `EMG-06` | Structural Anomaly | Vibration threshold exceeded | Throttle reduction and immediate landing | `Contingency_PrecautionaryLand` | 0.50 s | Monitor / Override |
| `EMG-07` | Flight Termination Cmd | Encrypted abort signal | Deploy parachute / instant motor cutoff | `Emergency_FlightTermination` | 0.02 s | Initiator |
"""


def _get_valid_mission_intent_content() -> str:
    return """| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent & Execution Plan |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |

# Tactical Mission Intent & Execution Plan

## 1. Commander's Intent & Operational Objectives
- **Operational Purpose:** Execute autonomous perimeter surveillance and situational awareness gathering.
- **Key Tasks:** Secure takeoff, transit surveillance corridor, stream sensor telemetry, return to base.
- **End State:** All survey waypoints verified, zero geofence breaches, safe recovery with >20% reserve energy.

## 2. Mission Essential Task List (METL)
| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `MET-01` | PreFlightSystemCheckout | Pre-launch power on | 100% PBIT pass in < 30 s | Automated BIT Log | `/// OperationalAllocation: [MET-01]` |
| `MET-02` | AutonomousIngressTransit | En-route nominal corridor | Cross-track error < 2.0 m | Flight Log Review | `/// OperationalAllocation: [MET-02]` |
| `MET-03` | AreaSurveillanceOrbit | On-station target orbit | Orbit radius 100 m +/- 5 m | Telemetry Stream | `/// OperationalAllocation: [MET-03]` |
| `MET-04` | AutonomousAreaSearch | Wide area search mode | 95% ground area coverage | Map Coverage Log | `/// OperationalAllocation: [MET-04]` |
| `MET-05` | SafeReturnAndDivert | Return to recovery point | Touchdown error < 1.0 m | Visual / RTK Log | `/// OperationalAllocation: [MET-05]` |
| `MET-06` | PostMissionDataOffload | Post-landing shutdown | Secure telemetry offload | Hash Verification | `/// OperationalAllocation: [MET-06]` |

## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Mission Area Coverage Ratio | A_covered / A_total | 0.90 | 0.99 | Dimensionless |
| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | 5.0 | 1.0 | m |
| MoP-02 | MoP | Telemetry Latency Bound | tau_transport | 50.0 | 10.0 | ms |

## 4. Threat & Electronic Warfare (EW) / Cyber Environment Matrix
| Threat ID | Threat Vector | Description | Severity | Autonomous Mitigation Rule |
| :--- | :--- | :--- | :--- | :--- |
| THR-01 | GNSS Spoofing / Jamming | Loss of carrier lock or pseudo-range jump | High | Revert to optical dead-reckoning and IMU integration |
| THR-02 | RF Link Interception / Jamming | C2 uplink SNR < 6 dB | Medium | Switch frequency-hopping channel or activate PACE alternate link |
| THR-03 | Cyber Ingress Attempt | Unauthorized packet on telemetry port | Critical | Immediate port isolation and cryptographic key cycle |

## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Priority / Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary** | Point-to-Point COFDM | 5.8 GHz ISM | 10.0 Mbps | 2.0 s | Video & High-rate Telemetry |
| **Alternate** | Cellular LTE / 5G VPN | Band 28 / 700 MHz | 2.0 Mbps | 3.0 s | Encrypted Cloud Relay |
| **Contingency** | 900 MHz FHSS Radio | 915 MHz ISM | 115.2 kbps | 5.0 s | Essential C2 Commands Only |
| **Emergency** | Satellite Iridium SBD | 1.6 GHz L-Band | 2.4 kbps | 10.0 s | Emergency Flight Termination & Geo-Beacon |

## 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks
- **ROE-01:** System shall not execute autonomous descent below $30\\text{ m}$ without positive ground radar clearance.
- **ROE-02:** Optical sensor laser pointer requires active human-in-the-loop (HITL) authorization key.
- **ROE-03:** Automated flight termination sequence requires two-person rule confirmation.

## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones
- **Primary Geofence Corridor:** Outer polygon bounding perimeter with $50\\text{ m}$ warning buffer.
- **Keep-Out Zones (Dynamic):** Populated area buffer circles ($R = 300\\text{ m}$) marked NO-FLY.
- **Separation Minima:** Maintain $150\\text{ m}$ vertical and $500\\text{ m}$ horizontal separation from non-cooperative targets.

## 8. Go/No-Go Decision Matrix
| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| GNG-01 | Pre-Launch | Battery State of Charge | >= 95.0% | Smart Battery BMS | Abort Launch if < 95% |
| GNG-02 | Pre-Launch | Wind Velocity | <= 12.0 m/s | Ground Anemometer | Hold Launch if > 12 m/s |
| GNG-03 | In-Flight | Navigation FOM | <= 2.0 | GPS Receiver HDOP | Initiate Loiter if > 2.0 |
| GNG-04 | In-Flight | Motor Temperature | <= 85.0 degC | ESC Thermistor | Divert to Base if > 85 degC |

## 9. Bingo Energy Mathematics & Secondary Divert Protocols
Bingo Energy dynamics model:
$$
\\begin{aligned}
E_{\\mathrm{bingo}}(t) &= E_{\\mathrm{return}}(\\mathbf{p}(t), \\mathbf{p}_{\\mathrm{dest}}) + E_{\\mathrm{divert}}(\\mathbf{p}_{\\mathrm{dest}}, \\mathbf{p}_{\\mathrm{alt}}) + E_{\\mathrm{reserve}} + E_{\\mathrm{contingency}} \\\\
E_{\\mathrm{reserve}} &\\ge 0.20 \\cdot E_{\\mathrm{capacity}}
\\end{aligned}
$$

| Energy Parameter | Symbol | Value | Units | Constraint Rule |
| :--- | :--- | :--- | :--- | :--- |
| Total Pack Capacity | E_capacity | 500.0 | kJ | Total nominal battery stored energy |
| Return Transit Energy | E_return | 150.0 | kJ | Energy required for primary return trajectory |
| Secondary Divert Energy | E_divert | 60.0 | kJ | Energy required to divert to secondary recovery site |
| Mandatory Statutory Reserve | E_reserve | 100.0 | kJ | 20.0% statutory reserve threshold |
| Contingency Buffer | E_contingency | 40.0 | kJ | Holding pattern & go-around reserve |
| Total Bingo Threshold | E_bingo | 350.0 | kJ | Critical return threshold (E_current <= E_bingo -> Divert) |

## 10. Gate 24 MissionTask Traceability Tags (Allocation Tags)
- `/// OperationalAllocation: [MET-01]`
- `/// OperationalAllocation: [MET-02]`
- `/// OperationalAllocation: [MET-03]`
- `/// OperationalAllocation: [MET-04]`
- `/// OperationalAllocation: [MET-05]`
- `/// OperationalAllocation: [MET-06]`
"""


class TestConOpsAndMissionIntentValidators(unittest.TestCase):
    """
    Comprehensive test suite for Gate 26: ConOps & Mission Intent Completeness Validators.
    """

    def test_clean_upstream_landing_zones_pass_gracefully(self):
        """Clean upstream workspace with empty landing zone returns zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            self.assertEqual(conops_val.validate(repo), [])
            self.assertEqual(mission_val.validate(repo), [])

    def test_valid_conops_and_mission_intent_passes_100_percent(self):
        """Fully compliant CONOPS.md (12 sections) and MISSION_INTENT.md (10 sections) return 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            conops_findings = conops_val.validate(repo)
            mission_findings = mission_val.validate(repo)

            self.assertEqual(conops_findings, [], f"Unexpected ConOps findings: {conops_findings}")
            self.assertEqual(mission_findings, [], f"Unexpected Mission Intent findings: {mission_findings}")

    def test_conops_missing_mandatory_section_fails(self):
        """Missing mandatory Section 6 in CONOPS.md triggers finding 'conops-section-missing'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Remove Section 6
            sec6_idx = content.find("## 6. 4D Operational Volume")
            sec7_idx = content.find("## 7. OMG UAF Operational Activity")
            truncated_content = content[:sec6_idx] + content[sec7_idx:]

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(truncated_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            missing_errors = [f for f in findings if f.rule_id == "conops-section-missing"]
            self.assertTrue(len(missing_errors) >= 1)
            self.assertIn("4D Operational Volume", str(missing_errors[0]))

    def test_conops_sora_grb_underdimensioned_fails(self):
        """SORA Ground Risk Buffer radius less than theoretical minimum triggers 'conops-sora-grb-underdimensioned'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Replace GRB radius 200.0 m with 20.0 m (under-dimensioned for h=120m, v_wind=15m/s -> min ~194m)
            content = content.replace("200.0 | m | Declared ground risk buffer radius", "20.0 | m | Declared ground risk buffer radius")

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            grb_errors = [f for f in findings if f.rule_id == "conops-sora-grb-underdimensioned"]
            self.assertTrue(len(grb_errors) >= 1)
            self.assertIn("Ground Risk Buffer", str(grb_errors[0]))

    def test_conops_incomplete_emergency_matrix_fails(self):
        """Emergency decision matrix with fewer than 7 canonical triggers triggers 'conops-emergency-matrix-incomplete'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Remove EMG-06 and EMG-07 rows
            content = content.replace("| `EMG-06` | Structural Anomaly | Vibration threshold exceeded | Throttle reduction and immediate landing | `Contingency_PrecautionaryLand` | 0.50 s | Monitor / Override |\n", "")
            content = content.replace("| `EMG-07` | Flight Termination Cmd | Encrypted abort signal | Deploy parachute / instant motor cutoff | `Emergency_FlightTermination` | 0.02 s | Initiator |\n", "")

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            emg_errors = [f for f in findings if f.rule_id == "conops-emergency-matrix-incomplete"]
            self.assertTrue(len(emg_errors) >= 1)
            self.assertIn("EMG-06", str(emg_errors[0]))
            self.assertIn("EMG-07", str(emg_errors[0]))

    def test_conops_malformed_table_fails(self):
        """Unclosed or broken CommonMark table triggers 'conops-table-malformed'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Break table formatting in section 8
            content = content.replace("| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |", "| Exchange ID | Source Node | Destination Node")

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            table_errors = [f for f in findings if f.rule_id in ("conops-table-malformed", "conops-section-missing")]
            self.assertTrue(len(table_errors) >= 1)

    def test_mission_intent_missing_mandatory_section_fails(self):
        """Missing mandatory Section 9 in MISSION_INTENT.md triggers 'mission-section-missing'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Remove Section 9
            sec9_idx = content.find("## 9. Bingo Energy Mathematics")
            sec10_idx = content.find("## 10. Gate 24 MissionTask Traceability")
            truncated_content = content[:sec9_idx] + content[sec10_idx:]

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(truncated_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            missing_errors = [f for f in findings if f.rule_id == "mission-section-missing"]
            self.assertTrue(len(missing_errors) >= 1)
            self.assertIn("Bingo Energy", str(missing_errors[0]))

    def test_mission_intent_bingo_reserve_insufficient_fails(self):
        """Statutory energy reserve below 20% triggers 'mission-bingo-reserve-insufficient'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Set total capacity = 500 kJ, but reserve = 20 kJ (4% < 20%)
            content = content.replace("100.0 | kJ | 20.0% statutory reserve threshold", "20.0 | kJ | 4.0% statutory reserve threshold")

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            bingo_errors = [f for f in findings if f.rule_id == "mission-bingo-reserve-insufficient"]
            self.assertTrue(len(bingo_errors) >= 1)
            self.assertIn("20.0%", str(bingo_errors[0]))

    def test_mission_intent_unallocated_metl_task_fails(self):
        """METL task missing Gate 24 allocation tag triggers 'mission-metl-unallocated'."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Strip allocation tag from MET-04 in Section 2 and Section 10
            content = content.replace("`/// OperationalAllocation: [MET-04]`", "—")
            content = content.replace("- `/// OperationalAllocation: [MET-04]`\n", "")

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            alloc_errors = [f for f in findings if f.rule_id == "mission-metl-unallocated"]
            self.assertTrue(len(alloc_errors) >= 1)
            self.assertIn("MET-04", str(alloc_errors[0]))

    def test_template_synthesis(self):
        """Verify synthesis of domain-neutral canonical templates."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_tpl_path = os.path.join(tmpdir, "CONOPS_CANONICAL_TEMPLATE.md")
            mission_tpl_path = os.path.join(tmpdir, "MISSION_INTENT_CANONICAL_TEMPLATE.md")

            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            self.assertTrue(conops_val.synthesize_canonical_template(conops_tpl_path))
            self.assertTrue(mission_val.synthesize_canonical_template(mission_tpl_path))

            self.assertTrue(os.path.exists(conops_tpl_path))
            self.assertTrue(os.path.exists(mission_tpl_path))

            with open(conops_tpl_path, "r", encoding="utf-8") as f:
                c_txt = f.read()
            with open(mission_tpl_path, "r", encoding="utf-8") as f:
                m_txt = f.read()

            for sec_idx in range(1, 13):
                self.assertIn(f"## {sec_idx}.", c_txt)

            for sec_idx in range(1, 11):
                self.assertIn(f"## {sec_idx}.", m_txt)

    def test_sora_math_helper(self):
        """Verify SORA Ground Risk Buffer calculation helper."""
        # h_max = 120.0 m, theta = 45.0 deg (tan=1.0), v_wind = 15.0 m/s
        # Expected: 120.0 * 1.0 + 15.0 * sqrt(2 * 120 / 9.80665) = 120 + 15 * 4.9469 = 194.20 m
        r_min = calculate_sora_grb_radius(h_max_m=120.0, theta_impact_deg=45.0, v_wind_max_mps=15.0)
        self.assertAlmostEqual(r_min, 194.20, delta=0.5)

    def test_bingo_energy_math_helper(self):
        """Verify Bingo energy reserve ratio calculation helper."""
        ratio = calculate_bingo_energy_reserve_ratio(total_capacity_j=500.0, reserve_energy_j=100.0)
        self.assertAlmostEqual(ratio, 0.20, delta=0.001)

    def test_aggregator_registration(self):
        """Verify ConopsCompletenessValidator and MissionIntentCompletenessValidator in AGGREGATING_VALIDATORS."""
        self.assertIn(ConopsCompletenessValidator, AGGREGATING_VALIDATORS)
        self.assertIn(MissionIntentCompletenessValidator, AGGREGATING_VALIDATORS)

    def test_cli_integration_clean_run(self):
        """Verify parity_auditor CLI executes cleanly on workspace with compliant ConOps and Mission Intent."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            schema_dir = os.path.join(tmpdir, "schema")
            pipeline_dir = os.path.join(tmpdir, ".pipeline", "logical-ui")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(pipeline_dir, exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

            # SysML model providing Gate 24 allocations
            sysml_content = """package SystemSSOT {
    doc /* /// OperationalAllocation: [OA-01, OA-02, OA-03, Phase_Startup, Phase_NominalExecution, Phase_DegradedMode, Phase_ContingencyFailsafe, Phase_SecureShutdown, Phase_MaintenanceMode] */
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            with open(os.path.join(pipeline_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
                f.write("""{
  "meta": {
    "upstream_repository": "acme/example-project"
  },
  "backlog_directories": {
    "schemas": "schema",
    "features": "docs/features",
    "epics": "docs/epics"
  },
  "target_directories": {
    "react": "",
    "flutter": ""
  },
  "tracker_rules": {}
}""")
            with open(os.path.join(pipeline_dir, "logical-layout.json"), "w", encoding="utf-8") as f:
                f.write("{}")

            cli_py = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "cli.py")
            import subprocess
            res = subprocess.run(
                [sys.executable, cli_py, "--workspace", tmpdir, "--schema-only"],
                capture_output=True,
                text=True
            )
            self.assertEqual(res.returncode, 0, f"CLI execution failed with stdout:\n{res.stdout}\nstderr:\n{res.stderr}")
            self.assertIn("ConOps & Mission Intent Completeness Audit", res.stdout)


if __name__ == "__main__":
    unittest.main()
