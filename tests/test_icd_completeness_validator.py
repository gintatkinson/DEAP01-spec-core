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
from parity_auditor.validators.icd_completeness_validator import ICDCompletenessValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS


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


def _setup_base_valid_workspace(tmpdir: str) -> None:
    """Helper to populate a clean, fully-compliant baseline workspace."""
    schema_dir = os.path.join(tmpdir, "schema")
    interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
    conops_dir = os.path.join(tmpdir, "docs", "conops")
    os.makedirs(schema_dir, exist_ok=True)
    os.makedirs(interfaces_dir, exist_ok=True)
    os.makedirs(conops_dir, exist_ok=True)

    with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
        f.write(_get_valid_conops_content())

    with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
        f.write(_get_valid_mission_intent_content())

    sysml_content = """package SystemSSOT {
    doc /* /// OperationalAllocation: [OA-01, OA-02, OA-03, Phase_Startup, Phase_NominalExecution, Phase_DegradedMode, Phase_ContingencyFailsafe, Phase_SecureShutdown, Phase_MaintenanceMode] */

    port def NavTelemetryPort {
        out item PrimaryVelocity : Float32;
        out item PitchAngle : Float32;
    }

    port def FlightControlPort {
        in item PrimaryVelocity : Float32;
        in item PitchAngle : Float32;
    }

    part def NavigationSubsystem {
        out port nav_out : NavTelemetryPort;
    }

    part def FlightControlSubsystem {
        in port fcc_in : FlightControlPort;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PrimaryVelocity;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PitchAngle;
}
"""
    with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
        f.write(sysml_content)

    icd01_content = """| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | System Interface Matrix & Topological Connectivity |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |
| **Type** | icd |
| **Interface Level** | Level 1C Logical Interface |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Level 1C: System Interface Matrix & Topological Connectivity

## 1. Executive Summary & Interface Scope
Topological connectivity matrix.

## 2. Subsystem Topological Connectivity Graph
```mermaid
flowchart TD
    subgraph NavigationSubsystem ["Navigation Subsystem"]
        P_NAV_OUT["PORT-NAV-OUT"]
    end
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        P_FCC_IN["PORT-FCC-IN"]
    end
    P_NAV_OUT -->|"CONN-01"| P_FCC_IN
```

## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |
| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-01` | `PORT-NAV-OUT` | `PORT-FCC-IN` | Continuous Stream | 10.0 | High | PrimaryVelocity, PitchAngle |
"""
    with open(os.path.join(interfaces_dir, "ICD_01_SYSTEM_INTERFACE_MATRIX.md"), "w", encoding="utf-8") as f:
        f.write(icd01_content)

    icd02_content = """| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #102 |
| **Title** | Master Signal Flow Dictionary & Safety Invariants |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |
| **Type** | icd |
| **Interface Level** | Level 1C Logical Interface |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Level 1C: Master Signal Flow Dictionary & Safety Invariants

## 1. Executive Summary & Signal Flow Overview
Overview of signal dictionary.

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |
"""
    with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
        f.write(icd02_content)


class TestICDCompletenessValidator(unittest.TestCase):
    # -------------------------------------------------------------------------
    # Baseline Happy & Existing Negative Path Tests
    # -------------------------------------------------------------------------

    def test_valid_icd_suite_passes(self):
        """Workspace with clean SysML model and fully matching ICD_01 and ICD_02 returns 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_dangling_port_detected(self):
        """Output port without destination connection flags finding with rule icd-dangling-port-detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            # Add dangling port to SysML and ICD_01
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            with open(schema_file, "a", encoding="utf-8") as f:
                f.write("""
    part def AuxSubsystem {
        out port aux_out : Float32;
    }
""")
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |\n| `PORT-AUX-OUT` | AuxSubsystem | aux_out | OUT | Float32 | 1 | PeriodicStream (100 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            dangling_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-dangling-port-detected"]
            self.assertTrue(len(dangling_errors) >= 1)
            self.assertEqual(dangling_errors[0].rule_id, "icd-dangling-port-detected")

    def test_unmapped_signal_detected(self):
        """SysML item flow not present in ICD_02_MASTER_SIGNAL_DICTIONARY.md flags icd-unmapped-signal-detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            with open(schema_file, "a", encoding="utf-8") as f:
                f.write("""
    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item UnmappedTelemetryFlow;
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            unmapped_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-unmapped-signal-detected"]
            self.assertTrue(len(unmapped_errors) >= 1)
            self.assertEqual(unmapped_errors[0].rule_id, "icd-unmapped-signal-detected")

    def test_missing_icd_suite_detected(self):
        """SysML model has ports/connections but docs/interfaces/ is missing flags icd-artifact-missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write("""package SystemSSOT {
    part def NavigationSubsystem {
        out port nav_out : Float32;
    }
}
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            missing_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-artifact-missing"]
            self.assertTrue(len(missing_errors) >= 1)
            self.assertEqual(missing_errors[0].rule_id, "icd-artifact-missing")

    def test_empty_workspace_skipped(self):
        """Workspace with no ports or schemas returns 0 findings cleanly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Malformed N² Matrix
    # -------------------------------------------------------------------------

    def test_adversarial_malformed_n2_matrix_missing_dest_header(self):
        """Adversarial test: N² matrix table is missing destination subsystem column header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Malform N² matrix: drop FlightControlSubsystem destination column header
            malformed_n2 = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem |
| :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** |
| **2. FlightControlSubsystem** | — |
"""
            content = content.replace(
                """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |""",
                malformed_n2
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)
            self.assertIn("missing destination column header", str(n2_errors[0]).lower())

    def test_adversarial_malformed_n2_matrix_missing_source_header(self):
        """Adversarial test: N² matrix table is missing source subsystem row header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Malform N² matrix: omit FlightControlSubsystem row
            malformed_n2 = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
"""
            content = content.replace(
                """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |""",
                malformed_n2
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)
            self.assertIn("missing source row header", str(n2_errors[0]).lower())


    def test_adversarial_malformed_n2_matrix_missing_entire_table(self):
        """Adversarial test: Multi-subsystem model where ICD_01 completely omits the N² matrix table."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Delete section 3 table
            target_section = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |"""
            content = content.replace(target_section, "## 3. Canonical N² Subsystem Interface Matrix\n(Omitted)")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Signal Dictionary Missing Mandatory Columns
    # -------------------------------------------------------------------------

    def test_adversarial_signal_dict_missing_safe_default_column(self):
        """Adversarial test: Signal dictionary is missing 'Safe Default Value' / 'Fault / Safe Value' column."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Remove Safe Default Value column
            old_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |"""

            new_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | `schema/model.sysml#L4` |"""

            content = content.replace(old_table, new_table)
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            col_errors = [f for f in findings if f.rule_id == "icd-missing-mandatory-column"]
            self.assertTrue(len(col_errors) >= 1)
            self.assertEqual(col_errors[0].detail.get("missing_column"), "Safe Default Value")

    def test_adversarial_signal_dict_missing_schema_citation_column(self):
        """Adversarial test: Signal dictionary is missing 'Schema Citation' column in table header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            old_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |"""

            new_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 |"""

            content = content.replace(old_table, new_table)
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            col_errors = [f for f in findings if f.rule_id == "icd-missing-mandatory-column"]
            self.assertTrue(len(col_errors) >= 1)
            self.assertEqual(col_errors[0].detail.get("missing_column"), "Schema Citation")

    def test_adversarial_signal_dict_row_tbd_values(self):
        """Adversarial test: Signal dictionary rows contain TBD for safe default, units, and citation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            content = content.replace("m/s", "TBD").replace("`schema/model.sysml#L3`", "TBD").replace("0.0", "TBD")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("icd-missing-units", rule_ids)
            self.assertIn("icd-missing-safe-default", rule_ids)
            self.assertIn("icd-missing-schema-citation", rule_ids)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Incompatible Port Data Types
    # -------------------------------------------------------------------------

    def test_adversarial_incompatible_port_types_boolean_to_float32(self):
        """Adversarial test: Incompatible port data types (Boolean connected to Float32) in ICD_01 port roster."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Set nav_out port type to Boolean while fcc_in is Float32
            content = content.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | Boolean | 1 | PeriodicStream (100 Hz) |"
            )
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | Float32 | 1 | PeriodicStream (100 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)
            self.assertIn("boolean", type_errors[0].detail.get("source_type", "").lower())
            self.assertIn("float32", type_errors[0].detail.get("dest_type", "").lower())

    def test_adversarial_incompatible_port_types_sysml_direct(self):
        """Adversarial test: SysML schema directly connects Boolean port to Float32 port."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            sysml_incompatible = """package SystemSSOT {
    part def NavigationSubsystem {
        out port nav_out : Boolean;
    }

    part def FlightControlSubsystem {
        in port fcc_in : Float32;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PrimaryVelocity;
}
"""
            with open(schema_file, "w", encoding="utf-8") as f:
                f.write(sysml_incompatible)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)

    def test_adversarial_incompatible_port_types_port_def_items(self):
        """Adversarial test: SysML port definitions contain incompatible item types (Boolean vs Float32)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            sysml_content = """package SystemSSOT {
    port def NavBoolPort {
        out item DiscreteFlag : Boolean;
    }

    port def FCCFloatPort {
        in item AnalogDemand : Float32;
    }

    part def NavigationSubsystem {
        out port nav_out : NavBoolPort;
    }

    part def FlightControlSubsystem {
        in port fcc_in : FCCFloatPort;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item DiscreteFlag;
}
"""
            with open(schema_file, "w", encoding="utf-8") as f:
                f.write(sysml_content)

            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace("NavTelemetryPort", "NavBoolPort").replace("FlightControlPort", "FCCFloatPort")
            c1 = c1.replace("PrimaryVelocity, PitchAngle", "DiscreteFlag")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("PrimaryVelocity", "DiscreteFlag").replace("Float32", "Boolean")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Incompatible Update Rates
    # -------------------------------------------------------------------------

    def test_adversarial_incompatible_update_rates_fast_publisher_slow_subscriber(self):
        """Adversarial test: Fast publisher (500 Hz) connected to slow subscriber (10 Hz) in ICD_01."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            content = content.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (500 Hz) |"
            )
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (10 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rate_errors = [f for f in findings if f.rule_id == "icd-incompatible-update-rate"]
            self.assertTrue(len(rate_errors) >= 1)
            self.assertIn("500", str(rate_errors[0].detail.get("publisher_rate", "")))
            self.assertIn("10", str(rate_errors[0].detail.get("subscriber_rate", "")))

    def test_adversarial_incompatible_update_rates_signal_rate_vs_subscriber(self):
        """Adversarial test: Signal update rate (200 Hz) exceeds destination port rate (20 Hz)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace("PeriodicStream (100 Hz)", "PeriodicStream (20 Hz)")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("100 Hz", "200 Hz")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rate_errors = [f for f in findings if f.rule_id == "icd-incompatible-update-rate"]
            self.assertTrue(len(rate_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Compound Fault Injection
    # -------------------------------------------------------------------------

    def test_adversarial_compound_fault_injection_all_faults_caught(self):
        """Adversarial test: Multiple simultaneous faults injected across N² matrix, columns, types, and rates."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)

            # 1. Malform N² matrix (remove dest header)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace(
                "| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |",
                "| Subsystem | 1. NavigationSubsystem |"
            )
            # Incompatible types on ports (Boolean to Float32)
            c1 = c1.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | Boolean | 1 | PeriodicStream (500 Hz) |"
            )
            c1 = c1.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | Float32 | 1 | PeriodicStream (10 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            # 2. Missing mandatory column in ICD_02 ('Safe Default Value')
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("Safe Default Value | ", "")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rule_ids = {f.rule_id for f in findings}

            # Verify 100% of injected fault categories are caught
            self.assertIn("icd-n2-matrix-malformed", rule_ids)
            self.assertIn("icd-missing-mandatory-column", rule_ids)
            self.assertIn("icd-port-type-incompatibility", rule_ids)
            self.assertIn("icd-incompatible-update-rate", rule_ids)

    # -------------------------------------------------------------------------
    # Aggregator and CLI Integration Tests
    # -------------------------------------------------------------------------

    def test_icd_completeness_validator_registered_in_aggregator(self):
        """Verify ICDCompletenessValidator is imported and registered in AGGREGATING_VALIDATORS in aggregator.py."""
        self.assertIn(ICDCompletenessValidator, AGGREGATING_VALIDATORS)

    def test_cli_integration_clean_run(self):
        """Verify parity_auditor CLI executes cleanly on valid workspace with --workspace and --schema-only."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            
            pipeline_dir = os.path.join(tmpdir, ".pipeline", "logical-ui")
            os.makedirs(pipeline_dir, exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
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
            self.assertIn("ICD Completeness & Signal Flow Parity Audit", res.stdout)
            self.assertIn("Level 1C ICD port connectivity, N² matrix, and signal dictionary verified.", res.stdout)


if __name__ == "__main__":
    unittest.main()
