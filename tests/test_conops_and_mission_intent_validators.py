import os
import re
import sys
import tempfile
import unittest
from unittest.mock import MagicMock

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
    lines = []
    # Header metadata
    lines.append("| Attribute | Value |")
    lines.append("| :--- | :--- |")
    lines.append("| **Title** | Concept of Operations (ConOps): Autonomous Cyber-Physical System Archetype |")
    lines.append("| **Version** | 1.0.0 |")
    lines.append("| **Date** | 2026-09-01 |")
    lines.append("")
    lines.append("# Concept of Operations (ConOps): Autonomous Cyber-Physical System Archetype")
    lines.append("")
    
    # Section 1
    lines.append("## 1. Scope & System Identification")
    lines.append("")
    lines.append("### 1.1 System Identification & Metamodel Governance")
    lines.append("The Autonomous Cyber-Physical System Archetype (`AutonomousSystemArchetype`) represents a modular, safety-critical autonomous platform.")
    lines.append("This document constitutes the canonical Level 1B Concept of Operations (ConOps) specification within the Digital Engineering Autonomous Pipeline (DEAP).")
    lines.append("It establishes the single source of truth for operational activities, operational envelopes, safety containment, and system modes.")
    lines.append("")
    lines.append("- **System Identifier:** `AutonomousSystemArchetype`")
    lines.append("- **Operational Domain:** `UAF::OperationalDomain::AutonomousMonitoringAndControl`")
    lines.append("- **Normative Baseline:** ISO/IEC/IEEE 29148:2018, INCOSE SE Handbook v5.0, OMG UAF v2.0")
    lines.append("- **Repository Scope:** Upstream Core Specification Suite")
    lines.append("")
    lines.append("### 1.2 System Operational Boundaries & Architecture")
    lines.append("The operational boundaries define the physical, electrical, and logical containment of the system in relation to external entities:")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TB")
    lines.append("    subgraph External_Boundary[\"External Operating Environment\"]")
    lines.append("        C2_Channel[\"PACE C2 Telemetry Channel\"]")
    lines.append("        GNSS_Constellation[\"Positioning & Timing Reference\"]")
    lines.append("        Environment[\"Environmental Dynamic Boundary\"]")
    lines.append("        Authority[\"Operational Control Authority\"]")
    lines.append("    end")
    lines.append("")
    lines.append("    subgraph System_Boundary[\"Autonomous Cyber-Physical System Core\"]")
    lines.append("        subgraph Compute_Subsystem[\"Fault-Tolerant Compute Core\"]")
    lines.append("            FCS[\"Flight & Guidance Controller\"]")
    lines.append("            Sensors[\"Perception & Navigation Fusion\"]")
    lines.append("            Watchdog[\"Hardware Safety Watchdog & Failsafe\"]")
    lines.append("        end")
    lines.append("")
    lines.append("        subgraph Power_Subsystem[\"Energy & Actuation Subsystem\"]")
    lines.append("            BMS[\"Smart Power Distribution & Storage\"]")
    lines.append("            Actuators[\"Distributed Real-Time Actuators\"]")
    lines.append("            Containment[\"Autonomous Physical Containment Device\"]")
    lines.append("        end")
    lines.append("    end")
    lines.append("")
    lines.append("    C2_Channel --- FCS")
    lines.append("    GNSS_Constellation --> Sensors")
    lines.append("    Environment --> Actuators")
    lines.append("    Authority --- FCS")
    lines.append("    Sensors --> FCS")
    lines.append("    FCS --> Actuators")
    lines.append("    FCS --> Watchdog")
    lines.append("    Watchdog --> Containment")
    lines.append("    BMS --> Compute_Subsystem")
    lines.append("    BMS --> Power_Subsystem")
    lines.append("```")
    lines.append("")
    lines.append("### 1.3 Stakeholder Roster & User Classes")
    lines.append("The operational lifecycle involves certified user classes and stakeholders operating under formal authority protocols:")
    lines.append("")
    lines.append("| User Class ID | Role Designation | Operational Responsibilities | Training & Qualification Level | HITL Authority Role |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| `UC-01` | System Supervisory Operator | Mission initialization, profile authorization, and real-time supervisory tracking | Level 3 Certified Autonomous Systems Operator | Primary HITL Supervisory Monitor / Override |")
    lines.append("| `UC-02` | Range Safety Officer | Range containment enforcement, emergency abort initiation, and corridor clearing | Level 4 Certified Range Safety Authority | Independent Flight Termination Authority |")
    lines.append("| `UC-03` | Payload / Sensor Specialist | Payload sensor calibration, real-time data stream parsing, and telemetry offload | Level 2 Sensor System Specialist | Informational / Payload Control |")
    lines.append("| `UC-04` | Maintenance Technician | O-Level / I-Level pre-flight BIT verification, battery hot-swap, and diagnostics | Certified Maintenance Engineer (CME) | Maintenance Mode Authority |")
    lines.append("| `UC-05` | Systems Integration Engineer | Cross-subsystem interface verification, schema compliance auditing, and calibration | Senior Systems Engineer | Engineering & Calibration Authority |")
    lines.append("| `UC-06` | Certification & Safety Auditor | Regulatory compliance inspection, safety case audit, and qualification assessment | Independent Safety Auditor | Audit & Verification Authority |")
    lines.append("| `UC-07` | Ground Support Crew | Physical placement, charging interface connection, and mechanical tie-down | Certified Ground Operations Specialist | Physical Handling Support |")
    lines.append("| `UC-08` | Telemetry Link Administrator | Frequency deconfliction, cryptographic key distribution, and link monitoring | Certified RF Network Administrator | Communications Infrastructure Role |")
    lines.append("")
    lines.append("### 1.4 Operational Assumptions & Constraints")
    lines.append("- Assumption 1: All operations occur within surveyed operational ranges carrying active spectrum clearance.")
    lines.append("- Assumption 2: Power infrastructure at ground staging sites provides continuous regulated charging.")
    lines.append("- Assumption 3: GNSS space vehicle geometry provides Dilution of Precision (DOP) $\le 2.5$ under nominal sky views.")
    lines.append("- Constraint 1: System shall maintain minimum statutory lateral separation from non-cooperative boundaries.")
    lines.append("- Constraint 2: All safety-critical state transitions execute deterministically with zero unhandled exceptions.")
    lines.append("")

    # Section 2
    lines.append("## 2. Normative Standards & Regulatory Baseline")
    lines.append("")
    lines.append("### 2.1 Applicable Normative Standards Register")
    lines.append("The engineering, verification, safety assurance, and operation of the platform strictly conform to the following standards baseline:")
    lines.append("")
    lines.append("| Standard ID | Issuing Body | Title / Baseline Description | Applicable Clauses & Focus Area |")
    lines.append("| :--- | :--- | :--- | :--- |")
    lines.append("| ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE | Systems and software engineering — Requirements engineering | §6.4.2 Concept of Operations & §6.4.3 Operational Concept |")
    lines.append("| INCOSE SEH v5.0 | INCOSE | Systems Engineering Handbook (5th Edition) | §3.3 Operational Concepts & §4.2 Requirements Engineering |")
    lines.append("| OMG UAF v1.2 / v2.0 | OMG | Unified Architecture Framework Specification | Operational Domain Views (Op-Pr, Op-Tx, Op-Is) |")
    lines.append("| NATO STANAG 4586 | NATO | Standard Interfaces of UAV Control System (UCS) for NATO Interoperability | Interoperability Data Link Interfaces (DLI §3.2) |")
    lines.append("| MIL-STD-882E | US DoD | Department of Defense Standard Practice: System Safety | Task 202: Operational Hazard Analysis (OHA §4.3) |")
    lines.append("| SAE ARP4761 / ARP4754A | SAE | Guidelines and Methods for Conducting Safety Assessment on Airborne Systems | §3: Functional Hazard Assessment (FHA) |")
    lines.append("| MIL-STD-1629A | US DoD | Procedures for Performing Failure Mode, Effects and Criticality Analysis | Method 101: Functional FMECA Analysis |")
    lines.append("| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment Methodology | Annex B: Ground Risk Class (GRC) & Ground Risk Buffer (GRB) |")
    lines.append("| RTCA DO-178C / DO-254 | RTCA | Software and Airborne Electronic Hardware Considerations in Safety Assurance | Safety Assurance Processes (DAL-A through DAL-C) |")
    lines.append("| MIL-STD-810H | US DoD | Environmental Engineering Considerations and Laboratory Tests | Environmental Qualification Envelopes (Method 501..514) |")
    lines.append("| IEEE 1588-2019 | IEEE | Precision Clock Synchronization Protocol for Networked Measurement Systems | Sub-microsecond deterministic time distribution |")
    lines.append("| NIST SP 800-82r3 | NIST | Guide to Industrial Control Systems (ICS) Security | §5: ICS Security Architecture and Telemetry Encryption |")
    lines.append("")
    lines.append("### 2.2 Clause-Level Allocation & Verification Compliance")
    lines.append("The normative obligations are systematically realized through formal verification mechanisms:")
    lines.append("")
    lines.append("| Obligation ID | Source Standard | Target Subsystem | Realization Mechanism | Verification Level |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| `OBL-CONOPS-01` | ISO/IEC/IEEE 29148 §6.4.2 | Entire System | 12-Section Canonical ConOps document structure | Gate 26 Automated Audit |")
    lines.append("| `OBL-CONOPS-02` | JARUS SORA v2.5 Annex B | Guidance Subsystem | Real-time Ground Risk Buffer dimension calculation | Gate 26 Mathematical Verification |")
    lines.append("| `OBL-CONOPS-03` | NATO STANAG 4586 §3.2 | Communications Core | Multi-tier PACE C2 link failover arbitration | Datalink Telemetry Verification |")
    lines.append("| `OBL-CONOPS-04` | MIL-STD-882E §4.3 | Safety Core | 7-Row Deterministic Emergency Decision Matrix | Automated Statechart Reachability |")
    lines.append("| `OBL-CONOPS-05` | OMG UAF v2.0 Op-Tx | Middleware Core | Strict schema-validated Op-Tx message taxonomy | Gate 24 Model Allocation Check |")
    lines.append("| `OBL-CONOPS-06` | RTCA DO-178C DAL-A | Watchdog Subsystem | Dedicated hardware watchdog with independent power rail | Hardware Hardware-in-the-Loop Test |")
    lines.append("")

    # Section 3
    lines.append("## 3. Current Situation & Deficiency Analysis (Predecessors)")
    lines.append("")
    lines.append("### 3.1 Predecessor Operational Baseline")
    lines.append("Predecessor operational configurations relied on legacy point-to-point analog telemetry links, manual piloting paradigms, and non-deterministic software controllers.")
    lines.append("These systems exhibited significant limitations in operational tempo, environmental resilience, and failsafe containment determinism.")
    lines.append("")
    lines.append("### 3.2 Detailed Deficiency Taxonomy")
    lines.append("Operational field reviews and risk evaluations identified six structural deficiencies in predecessor architectures:")
    lines.append("1. **Single-Point C2 Link Vulnerabilities:** Legacy systems lacked multi-tier PACE failover, causing complete loss of situational awareness during RF degradation.")
    lines.append("2. **Under-Dimensioned Risk Containment:** Ground Risk Buffers were calculated using heuristic approximations rather than formal SORA v2.5 kinetic equations.")
    lines.append("3. **Excessive Emergency Latencies:** Emergency triggers required manual operator intervention, resulting in response latencies exceeding 5.0 seconds.")
    lines.append("4. **Lack of Model Traceability:** Operational requirements were documented in informal prose without bidirectional Gate 24 allocation tags to SysML v2 models.")
    lines.append("5. **Unstructured Maintenance Protocols:** Lack of structured O/I/D maintenance level task allocation resulted in extended diagnostic downtime.")
    lines.append("6. **Sub-Optimal Energy Management:** Battery reserve thresholds were static and did not dynamically compute wind-compensated return energy dynamics.")
    lines.append("")
    lines.append("### 3.3 Predecessor vs Modernized Target Comparison")
    lines.append("The operational capabilities of predecessor architectures versus the modernized archetype are contrasted below:")
    lines.append("")
    lines.append("| Capability Dimension | Predecessor Operational Baseline | Modernized Target Archetype | Improvement Factor |")
    lines.append("| :--- | :--- | :--- | :--- |")
    lines.append("| Emergency Reaction Time | 5.0 s (Manual HITL Response) | <= 0.05 s (Automated Hardware Watchdog) | 100x Latency Reduction |")
    lines.append("| C2 Datalink Availability | Simplex 2.4 GHz (98.2% availability) | Quad-Tier PACE Infrastructure (99.999% availability) | 5-Nines Resilient Reachability |")
    lines.append("| Containment Buffer Accuracy | Heuristic Fixed 50 m Buffer | SORA v2.5 Aerodynamic Calculation (R_GRB = 200.0 m) | Zero Boundary Breaches |")
    lines.append("| Telemetry Schema Validation | Unchecked Bitfields | Strongly-Typed CommonMark & SysML v2 Taxonomy | 100% Type-Safe Interoperability |")
    lines.append("| Turnaround Inspection Time | 45 minutes (Manual Checklist) | <= 5 minutes (Automated O-Level BIT Scan) | 9x Faster Sortie Turnaround |")
    lines.append("")

    # Section 4
    lines.append("## 4. Operational Justification & Priority Matrix (Trade-Offs)")
    lines.append("")
    lines.append("### 4.1 Mission Drivers & Value Propositions")
    lines.append("The Autonomous Cyber-Physical System Archetype resolves predecessor deficiencies by introducing deterministic safety architectures, verified mathematical containment, and structured UAF activity modeling.")
    lines.append("The core mission value propositions include:")
    lines.append("- Uncompromised public safety through formally verified 4D ground risk containment.")
    lines.append("- Continuous situational awareness across remote and contested operating environments.")
    lines.append("- Standardized digital engineering lifecycle compatibility conforming to DEAP specifications.")
    lines.append("")
    lines.append("### 4.2 Pugh Decision Matrix & Architectural Trade-Off Analysis")
    lines.append("To determine the optimal system architecture, a multi-criteria Pugh Decision Matrix analysis was performed.")
    lines.append("The candidates evaluated against the legacy baseline (Datum) include:")
    lines.append("- **Baseline (Datum):** Legacy Simplex Architecture with Manual Override.")
    lines.append("- **Candidate A:** Dual-Redundant Centralized RTOS Architecture with Software Watchdog.")
    lines.append("- **Candidate B (Selected):** Distributed Fault-Tolerant Core with Hardware Safety Watchdog and Multi-Tier PACE C2.")
    lines.append("- **Candidate C:** Triple Modular Redundancy (TMR) Core with Ballistic Recovery Subsystem.")
    lines.append("")
    lines.append("The multi-criteria Pugh score $S_j(w)$ and sensitivity equations $\\frac{\\partial S_j}{\\partial w_i}$ are formulated as:")
    lines.append("")
    lines.append("$$")
    lines.append("\\begin{aligned}")
    lines.append("S_j(w) &= \\sum_{i=1}^{M} w_i \\cdot c_{ij} \\\\")
    lines.append("\\sum_{i=1}^{M} w_i &= 1.0 \\\\")
    lines.append("\\frac{\\partial S_j}{\\partial w_i} &= c_{ij}")
    lines.append("\\end{aligned}")
    lines.append("$$")
    lines.append("")
    lines.append("| Evaluation Criterion (i) | Weight (w_i) | Baseline (Datum) | Candidate Architecture A | Candidate Architecture B (Selected) | Candidate Architecture C |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| Operational Reliability & MTBCF | 0.20 | 0 (Datum) | +1 (0.20) | +2 (0.40) | +2 (0.40) |")
    lines.append("| Deterministic Safety Containment | 0.25 | 0 (Datum) | +1 (0.25) | +2 (0.50) | +2 (0.50) |")
    lines.append("| Multi-Domain Threat Resilience | 0.15 | 0 (Datum) | 0 (0.00) | +2 (0.30) | +1 (0.15) |")
    lines.append("| Turnaround & O-Level Logistics | 0.15 | 0 (Datum) | +1 (0.15) | +2 (0.30) | -1 (-0.15) |")
    lines.append("| Mass Budget & Energy Efficiency | 0.15 | 0 (Datum) | 0 (0.00) | +1 (0.15) | -2 (-0.30) |")
    lines.append("| Lifecycle Cost & Complexity | 0.10 | 0 (Datum) | -1 (-0.10) | 0 (0.00) | -2 (-0.20) |")
    lines.append("| **Weighted Total Score S_j(w)** | **1.00** | **0.00** | **+0.50** | **+1.65** | **+0.40** |")
    lines.append("")
    lines.append("### 4.3 Sensitivity Analysis & Gradient Robustness")
    lines.append("Sensitivity analysis demonstrates that Candidate Architecture B remains optimal across all weight variations.")
    lines.append("Even when cost weighting increases to $w_6 = 0.30$, Candidate B maintains a score advantage of $> 0.85$ over Candidate A and Candidate C.")
    lines.append("The partial derivative vector $\\nabla S_B = [2, 2, 2, 2, 1, 0]^T$ proves strict dominance across all primary safety criteria.")
    lines.append("")

    # Section 5
    lines.append("## 5. Operational Modes & Lifecycle Stages")
    lines.append("")
    lines.append("### 5.1 Formal Operational Lifecycle Stages Across \\Phi_{\\mathrm{lifecycle}}")
    lines.append("The system lifecycle is partitioned into six formal operational stages conforming to ISO/IEC/IEEE 29148:2018:")
    lines.append("- **Phase_Startup:** Power-on Built-In-Test (PBIT), sensor calibration, cryptographic key verification, and geofence initialization.")
    lines.append("- **Phase_NominalExecution:** Automated waypoint tracking, sensor payload operation, situational telemetry relay, and guidance.")
    lines.append("- **Phase_DegradedMode:** Subsystem anomaly handling, simplex sensor fallback, and reduced performance envelope operation.")
    lines.append("- **Phase_ContingencyFailsafe:** Emergency contingency execution, autonomous lost-link loiter, return-to-base, or divert routing.")
    lines.append("- **Phase_SecureShutdown:** Controlled touchdown, power bus de-energization, diagnostic blackbox offload, and cryptographic clearing.")
    lines.append("- **Phase_MaintenanceMode:** Diagnostic interface connection, I-Level/D-Level calibration, firmware flashing, and hardware maintenance.")
    lines.append("")
    lines.append("### 5.2 Mode Transition Rules & Preemption Matrix")
    lines.append("The operational state transitions are governed by deterministic guard conditions and timing bounds:")
    lines.append("")
    lines.append("| Source Operational Mode | Triggering Event / Anomaly | Target Operational Mode | Guard Condition / State Predicate | Max Latency Deadline |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| Phase_Startup | PBIT_Success_Event | Phase_NominalExecution | All health flags pass and C2 link active | <= 100 ms |")
    lines.append("| Phase_NominalExecution | Sensor_Disparity_Event | Phase_DegradedMode | Disparity persistence > 50 ms | <= 50 ms |")
    lines.append("| Phase_NominalExecution | Critical_Failsafe_Trigger | Phase_ContingencyFailsafe | EMG-01..07 active trigger | <= 20 ms |")
    lines.append("| Phase_DegradedMode | C2_Heartbeat_Timeout | Phase_ContingencyFailsafe | Heartbeat age > 5.0 s | <= 50 ms |")
    lines.append("| Phase_ContingencyFailsafe | Safe_Touchdown_Verified | Phase_SecureShutdown | Ground contact verified by weight sensor | <= 100 ms |")
    lines.append("| Phase_SecureShutdown | Maintenance_Interface_Active | Phase_MaintenanceMode | Ground interlock enabled | <= 500 ms |")
    lines.append("")

    # Section 6
    lines.append("## 6. 4D Operational Volume & SORA Ground Risk Buffer Mathematics")
    lines.append("")
    lines.append("### 6.1 4D Operational Volume Mathematical Formulation")
    lines.append("The 4D operational volume $V_{\\mathrm{4D}}$ comprises the nominal flight geometry volume $V_{\\mathrm{FlightGeometry}}$, the contingency volume $V_{\\mathrm{ContingencyVolume}}$, and the Ground Risk Buffer $V_{\\mathrm{GRB}}$ calculated in accordance with JARUS SORA v2.5 Annex B:")
    lines.append("")
    lines.append("$$")
    lines.append("\\begin{aligned}")
    lines.append("V_{\\mathrm{4D}} &= V_{\\mathrm{FlightGeometry}} \\cup V_{\\mathrm{ContingencyVolume}} \\cup V_{\\mathrm{GRB}} \\\\")
    lines.append("R_{\\mathrm{GRB}} &= h_{\\mathrm{max}} \\cdot \\tan(\\theta_{\\mathrm{impact}}) + v_{\\mathrm{wind,max}} \\cdot \\sqrt{\\frac{2 h_{\\mathrm{max}}}{g}} + d_{\\mathrm{glide,max}}")
    lines.append("\\end{aligned}")
    lines.append("$$")
    lines.append("")
    lines.append("### 6.2 SORA Ground Risk Buffer Parameter Register")
    lines.append("The physical and operational parameters governing containment volume dimensioning are defined in the following table:")
    lines.append("")
    lines.append("| Parameter | Symbol | Value | Units | Description |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| Max Altitude / Ceiling | h_max | 120.0 | m | Maximum operating ceiling above reference surface |")
    lines.append("| Impact Angle | theta_impact | 45.0 | deg | Worst-case operational trajectory impact angle (1:1 SORA ratio) |")
    lines.append("| Max Wind Speed | v_wind_max | 15.0 | m/s | Maximum operational environmental wind speed limit |")
    lines.append("| Gravitational Accel | g | 9.80665 | m/s^2 | Standard gravitational acceleration constant |")
    lines.append("| Maximum Glide Distance | d_glide_max | 0.0 | m | Maximum unpowered lateral displacement margin (simplex rotor) |")
    lines.append("| Ground Risk Buffer Radius | R_GRB | 200.0 | m | Declared ground risk buffer containment radius |")
    lines.append("| Terminal Velocity | v_terminal | 25.0 | m/s | Estimated unpowered descent terminal velocity |")
    lines.append("| Impact Kinetic Energy | E_impact | 1562.5 | J | Kinetic energy at operational boundary impact (m=5.0 kg) |")
    lines.append("")
    lines.append("### 6.3 SORA Compliance Assessment")
    lines.append("The theoretical minimum buffer radius for $h_{\\mathrm{max}} = 120.0\\text{ m}$ and $v_{\\mathrm{wind,max}} = 15.0\\text{ m/s}$ is $R_{\\mathrm{calc}} = 194.20\\text{ m}$.")
    lines.append("The declared Ground Risk Buffer $R_{\\mathrm{GRB}} = 200.0\\text{ m}$ exceeds the theoretical floor, guaranteeing containment compliance.")
    lines.append("")

    # Section 7
    lines.append("## 7. OMG UAF Operational Activity Taxonomy")
    lines.append("")
    lines.append("### 7.1 Doctrinal Operational Activities (Op-Pr View)")
    lines.append("In conformance with OMG UAF v2.0 Operational Performer (Op-Pr) view, operational tasks are categorized into structured, discrete activities carrying formal Gate 24 allocation tags:")
    lines.append("")
    lines.append("| Activity ID | Activity Name | Description | Gate 24 Allocation Tag |")
    lines.append("| :--- | :--- | :--- | :--- |")
    lines.append("| OA-01 | PreFlightSystemInitialization | Executes automated power-on Built-In-Tests, peripheral health diagnostics, and calibration | `/// OperationalAllocation: [OA-01]` |")
    lines.append("| OA-02 | ExecuteTrajectoryGuidance | Performs closed-loop waypoint tracking, speed governing, and 4D trajectory following | `/// OperationalAllocation: [OA-02]` |")
    lines.append("| OA-03 | MultiModalSensorFusion | Ingests IMU, GNSS, optical odometry, and air data telemetry to produce optimal state estimates | `/// OperationalAllocation: [OA-03]` |")
    lines.append("| OA-04 | ActivePerimeterMonitoring | Continuously compares current estimated state against 4D operational volume boundaries | `/// OperationalAllocation: [OA-04]` |")
    lines.append("| OA-05 | HealthAndContingencyManagement | Evaluates subsystem telemetry against anomaly rosters and triggers emergency recovery | `/// OperationalAllocation: [OA-05]` |")
    lines.append("| OA-06 | TelemetryExchangeAndRelay | Streams high-integrity downlink telemetry and receives encrypted uplink command packets | `/// OperationalAllocation: [OA-06]` |")
    lines.append("| OA-07 | AutonomousRecoveryExecution | Manages precision descent, terrain clearance verification, and touchdown containment | `/// OperationalAllocation: [OA-07]` |")
    lines.append("| OA-08 | PostFlightDiagnosticOffload | Performs secure non-volatile flash logging offload, cryptographic clearing, and power-off | `/// OperationalAllocation: [OA-08]` |")
    lines.append("| OA-09 | DynamicResourceManagement | Continuously evaluates remaining battery energy against return and divert dynamics | `/// OperationalAllocation: [OA-09]` |")
    lines.append("| OA-10 | EnvironmentalEnvelopeSupervision | Monitors ambient temperature, wind gusts, and precipitation against operating limits | `/// OperationalAllocation: [OA-10]` |")
    lines.append("| OA-11 | InterlockVerificationAndArming | Confirms safety interlock states prior to propulsion and actuator energization | `/// OperationalAllocation: [OA-11]` |")
    lines.append("| OA-12 | GroundAuditLogSynchronization | Transfers cryptographic verification hashes and sensor diagnostic traces to ground | `/// OperationalAllocation: [OA-12]` |")
    lines.append("")

    # Section 8
    lines.append("## 8. Operational Information Exchange (Op-Tx) Matrix")
    lines.append("")
    lines.append("### 8.1 Operational Information Exchange Matrix (Op-Tx View)")
    lines.append("The inter-subsystem and external information exchange flows are formalized in the following Op-Tx Matrix:")
    lines.append("")
    lines.append("| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| OpTx-01 | PrimarySensors | FlightController | FilteredSensorState | 100 Hz | 5 ms | High (DAL-A) |")
    lines.append("| OpTx-02 | FlightController | ActuatorSubsystem | ActuatorCommandVector | 200 Hz | 2.5 ms | High (DAL-A) |")
    lines.append("| OpTx-03 | SafetyWatchdog | FlightController | HeartbeatPulseAndStatus | 50 Hz | 10 ms | Critical (DAL-A) |")
    lines.append("| OpTx-04 | SafetyWatchdog | ContainmentDevice | ContainmentTriggerCmd | Event | 1 ms | Catastrophic (DAL-A) |")
    lines.append("| OpTx-05 | FlightController | GroundStation | TelemetryStateStream | 20 Hz | 50 ms | Medium (DAL-B) |")
    lines.append("| OpTx-06 | GroundStation | FlightController | SupervisoryCommandVector | 10 Hz | 100 ms | High (DAL-B) |")
    lines.append("| OpTx-07 | BatteryBMS | FlightController | StateOfChargeAndThermal | 10 Hz | 50 ms | High (DAL-B) |")
    lines.append("| OpTx-08 | DiagnosticPort | MaintenanceConsole | NonVolatileAuditLog | Batch | 500 ms | Low (DAL-C) |")
    lines.append("| OpTx-09 | NavigationFilter | SafetyWatchdog | StateBoundaryProximityData | 50 Hz | 10 ms | High (DAL-A) |")
    lines.append("| OpTx-10 | AirDataUnit | FlightController | DynamicPressureAndGustRate | 50 Hz | 10 ms | High (DAL-B) |")
    lines.append("| OpTx-11 | OpticalTracker | GuidanceFilter | RelativeTargetBearingVector | 30 Hz | 33 ms | Medium (DAL-B) |")
    lines.append("| OpTx-12 | RangeSafetyTerminal | WatchdogSubsystem | EncryptedAbortTriggerFrame | Event | 10 ms | Catastrophic (DAL-A) |")
    lines.append("")
    lines.append("### 8.2 Operational Interaction Sequence & Exchange Protocol")
    lines.append("The interaction sequence between operators, flight controller, safety watchdog, and actuation devices is specified below:")
    lines.append("")
    lines.append("```mermaid")
    lines.append("sequenceDiagram")
    lines.append("    autonumber")
    lines.append("    participant GCS as Ground Station (UC-01)")
    lines.append("    participant FC as Flight Controller Core")
    lines.append("    participant WD as Safety Watchdog (DAL-A)")
    lines.append("    participant ACT as Actuator Subsystem")
    lines.append("    participant CD as Containment Device")
    lines.append("")
    lines.append("    GCS->>FC: Upload Authorized Mission Plan")
    lines.append("    FC->>WD: Arm Safety Watchdog & Geofence")
    lines.append("    WD-->>FC: Watchdog Armed (Acknowledge)")
    lines.append("    FC->>ACT: Initialize Actuator Demand")
    lines.append("    loop Operational Execution (100 Hz)")
    lines.append("        FC->>ACT: Stream Dynamic Demand")
    lines.append("        FC->>WD: Periodic Liveness Heartbeat")
    lines.append("        WD-->>FC: Heartbeat Valid")
    lines.append("        FC->>GCS: Stream Downlink Telemetry")
    lines.append("    end")
    lines.append("    alt Anomaly / Emergency Trigger (EMG-07 Abort)")
    lines.append("        GCS->>FC: Encrypted Emergency Abort Command")
    lines.append("        FC->>WD: Trigger Safe State Containment")
    lines.append("        WD->>ACT: Inhibit Power Bus")
    lines.append("        WD->>CD: Actuate Physical Containment Device")
    lines.append("        CD-->>GCS: Safe State Impact Containment Confirmed")
    lines.append("    end")
    lines.append("```")
    lines.append("")

    # Section 9
    lines.append("## 9. Operational Environments & Constraints")
    lines.append("")
    lines.append("### 9.1 Environmental Limits Register")
    lines.append("The system is qualified for operation across rigorous environmental envelopes conforming to MIL-STD-810H and statutory operating constraints:")
    lines.append("")
    lines.append("| Environmental Dimension | Nominal Range | Extreme Limit | Verification Standard | Operational Constraint Rule |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| Ambient Operating Temperature | -20.0 degC to +45.0 degC | -30.0 degC to +55.0 degC | MIL-STD-810H Method 501.7/502.7 | Thermal management active at T < 0 degC |")
    lines.append("| Environmental Ingress Rating | IP65 (Sealed) | IP67 (Avionics Bay) | IEC 60529 | Dust-tight, resistant to heavy water spray |")
    lines.append("| Operational Wind Velocity | 0.0 to 10.0 m/s | 15.0 m/s | MIL-STD-810H Method 514.8 | Operations hold if gust > 15.0 m/s |")
    lines.append("| Continuous Precipitation | 0.0 mm/hr | 10.0 mm/hr | MIL-STD-810H Method 506.6 | Moderate precipitation operations certified |")
    lines.append("| Electromagnetic / RF Resilience | Nominal RF Spectrum | High-EMI Degraded | MIL-STD-461G | Frequency-hopping fallback under jamming |")
    lines.append("| Spatial Staging Footprint | 2.0 m x 2.0 m | 3.0 m x 3.0 m | Standard Field Range | Clear horizontal clearance required |")
    lines.append("| Shock & Vibration Tolerance | Operational Profile A | 20g / 11 ms Sawtooth | MIL-STD-810H Method 516.8 | Airframe structural integrity verified |")
    lines.append("| Relative Humidity Envelope | 5% to 90% Non-Condensing | 95% Condensing | MIL-STD-810H Method 507.6 | Conformal coating on all circuit boards |")
    lines.append("| Atmospheric Pressure Ceiling | 1013.25 hPa | 700.0 hPa (FL100) | MIL-STD-810H Method 500.6 | Barometric altitude calibration required |")
    lines.append("| Solar Radiation Flux | 0 to 1120 W/m^2 | 1200 W/m^2 | MIL-STD-810H Method 505.7 | Thermal shielding over optical sensor ports |")
    lines.append("")

    # Section 10
    lines.append("## 10. Multi-Threaded Operational Scenarios")
    lines.append("")
    lines.append("### 10.1 Scenario 1: Nominal Autonomous Mission Execution")
    lines.append("- **Thread Context:** Nominal automated staging, ingress corridor traversal, area monitoring, and precision recovery.")
    lines.append("- **Step 1:** Pre-flight power-on BIT completes in 20 s with 100% component pass confirmation.")
    lines.append("- **Step 2:** Automated ascent to operational ceiling $h = 100.0\\text{ m}$ at climb rate $v_z = 3.0\\text{ m/s}$.")
    lines.append("- **Step 3:** Closed-loop traversal of 12 mission waypoints maintaining cross-track error $\\Delta d \\le 1.0\\text{ m}$.")
    lines.append("- **Step 4:** Continuous payload telemetry streaming at 10.0 Mbps over primary COFDM datalink.")
    lines.append("- **Step 5:** Automated perimeter surveillance sweeps executed with zero exclusion zone excursions.")
    lines.append("- **Step 6:** Continuous battery health and thermal status monitoring at 10 Hz.")
    lines.append("- **Step 7:** Guidance computer calculates Bingo return energy and initiates egress trajectory.")
    lines.append("- **Step 8:** Egress transit completed along clear return corridor maintaining altitude $h = 100.0\\text{ m}$.")
    lines.append("- **Step 9:** Autonomous vertical descent initiated over primary surveyed recovery zone.")
    lines.append("- **Step 10:** Touchdown confirmed by weight-on-wheels sensor; motors shut down with remaining energy $> 25\\%$.")
    lines.append("")
    lines.append("### 10.2 Scenario 2: Primary Navigation Degradation & Inertial Fallback")
    lines.append("- **Thread Context:** Loss of primary GNSS carrier lock during mid-mission transit.")
    lines.append("- **Step 1:** Navigation filter detects pseudo-range jump and FOM degradation exceeding 2.0.")
    lines.append("- **Step 2:** System automatically transitions from `Phase_NominalExecution` to `Phase_DegradedMode`.")
    lines.append("- **Step 3:** Optical odometry and dead-reckoning estimator assume primary navigation authority within 50 ms.")
    lines.append("- **Step 4:** Flight guidance throttles forward speed and initiates return-to-base along reciprocal ingress heading.")
    lines.append("- **Step 5:** Safety watchdog monitors dead-reckoning drift rates against spatial containment buffer.")
    lines.append("- **Step 6:** Ground station alerts operator of degraded mode state with automated telemetry update.")
    lines.append("- **Step 7:** Optical feature tracking locates primary recovery marker during terminal descent.")
    lines.append("- **Step 8:** System lands safely within secondary containment zone with zero geofence excursions.")
    lines.append("")
    lines.append("### 10.3 Scenario 3: Lost C2 Link Contingency & Autonomous RTB")
    lines.append("- **Thread Context:** Complete RF link loss exceeding heartbeat timeout threshold $\\tau_{\\mathrm{loss}} = 5.0\\text{ s}$.")
    lines.append("- **Step 1:** Primary C2 heartbeat timer expires; system switches automatically to Alternate LTE channel.")
    lines.append("- **Step 2:** Alternate channel unacknowledged after 3.0 s; system enters `Phase_ContingencyFailsafe`.")
    lines.append("- **Step 3:** Autonomous lost-link loiter pattern executed for 30.0 s at current station altitude.")
    lines.append("- **Step 4:** Autonomous climb to safe clearance altitude ($h = 120.0\\text{ m}$) and direct return to base.")
    lines.append("- **Step 5:** Autonomous vertical descent and engine shutdown at primary recovery pad.")
    lines.append("")
    lines.append("### 10.4 Scenario 4: Dynamic Divert to Secondary Recovery Site")
    lines.append("- **Thread Context:** Primary recovery point obstructed by unpredicted severe localized weather.")
    lines.append("- **Step 1:** Environmental monitor reports gust velocity exceeding 15.0 m/s at primary landing site.")
    lines.append("- **Step 2:** Guidance computer computes remaining energy against secondary divert destination.")
    lines.append("- **Step 3:** Dynamic Bingo evaluation verifies $E_{\\mathrm{current}} \\ge E_{\\mathrm{divert}} + E_{\\mathrm{reserve}}$.")
    lines.append("- **Step 4:** Autonomous divert trajectory engaged with positive terrain separation $> 50.0\\text{ m}$.")
    lines.append("- **Step 5:** Safe precision touchdown executed at secondary surveyed recovery coordinate.")
    lines.append("")

    # Section 11
    lines.append("## 11. Maintenance & Sustainment Concepts (O/I/D Maintenance)")
    lines.append("")
    lines.append("### 11.1 Three-Tier Maintenance Allocation Model")
    lines.append("The maintenance and sustainment concept follows a three-tier doctrinal structure:")
    lines.append("")
    lines.append("| Maintenance Tier | Organizational Level | Maintenance Tasks & Work Scope | Tooling & Equipment | Interval / Trigger |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| **O-Level** (Organizational) | Field Staging Site | Pre/post-operation inspections, modular battery hot-swap, visual check, PBIT review | Field Diagnostic Tablet, Standard Hand Tools | Every mission turnaround |")
    lines.append("| **I-Level** (Intermediate) | Mobile Support Unit | Actuator calibration, sensor alignment, modular LRU swap, harness continuity test | Automated Test Bench, Calibrated Fixtures | 100 operating hours / unscheduled fault |")
    lines.append("| **D-Level** (Depot) | Central Overhaul Facility | Airframe structural overhaul, composite NDI inspection, safety computer recertification | Ultrasonic NDI, Environmental Chamber | 500 operating hours / major overhaul |")
    lines.append("| **O-Level Logistics** | Supply Point | Spare modular battery packs, quick-release fasteners, prop adapters | Portable Storage Cases | Continuous readiness |")
    lines.append("| **I-Level Diagnostics** | Calibration Lab | Inertial measurement unit bias calibration and optical bench alignment | Precision Rate Table | 250 operating hours |")
    lines.append("| **D-Level Recertification**| Qualification Facility | Environmental stress screening and full software regression test | Hardware-in-the-Loop Simulator | 1000 operating hours |")
    lines.append("")
    lines.append("### 11.2 Spares Provisioning & Diagnostic Offload Strategy")
    lines.append("- Spares Provisioning: Critical line-replaceable units (LRUs) are provisioned in modular, field-swappable enclosures.")
    lines.append("- Diagnostic Telemetry: High-speed USB-C and optical diagnostic interfaces enable 100 MB/s blackbox audit log extraction.")
    lines.append("- Preventive Maintenance: Firmware hash checks and component lifetime counters are audited prior to every operating window.")
    lines.append("")

    # Section 12
    lines.append("## 12. 7-Row Emergency Decision & Contingency Matrix")
    lines.append("")
    lines.append("### 12.0 Canonical 7-Row Emergency Matrix")
    lines.append("The following matrix defines the canonical deterministic emergency triggers (`EMG-01` through `EMG-07`):")
    lines.append("")
    lines.append("| Trigger ID | Contingency Trigger | Detection Mechanism | Automated Containment Action | Failsafe State | Max Response Time | HITL Role |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| `EMG-01` | Lost C2 Link | Heartbeat loss > 5.0 s | Execute autonomous lost-link loiter / return | `Contingency_LostLinkReturn` | 0.50 s | Monitor / Override |")
    lines.append("| `EMG-02` | GNSS Navigation Loss | FOM > 3.0 or RAIM Alert | Switch to dead-reckoning / optical odometry | `Contingency_DeadReckoning` | 0.10 s | Monitor / Override |")
    lines.append("| `EMG-03` | Propulsion Failure | RPM drop / Over-current | Execute glide to nearest secondary divert site | `Contingency_EmergencyGlide` | 0.05 s | Informed |")
    lines.append("| `EMG-04` | Critical Sensor Fault | Cross-channel disparity | Revert to simplex failsafe sensor mode | `Degraded_SensorFailsafe` | 0.05 s | Monitor |")
    lines.append("| `EMG-05` | Geofence Breach Alert | Boundary proximity < 50 m | Execute emergency turnaround maneuver | `Contingency_GeofenceContainment` | 0.20 s | Monitor / Override |")
    lines.append("| `EMG-06` | Structural Anomaly | Vibration threshold exceeded | Throttle reduction and immediate landing | `Contingency_PrecautionaryLand` | 0.50 s | Monitor / Override |")
    lines.append("| `EMG-07` | Flight Termination Cmd | Encrypted abort signal | Deploy parachute / instant motor cutoff | `Emergency_FlightTermination` | 0.02 s | Initiator |")
    lines.append("")
    lines.append("### 12.1 Failsafe State Transition Semantics & Timing Guarantees")
    lines.append("$$")
    lines.append("\\begin{aligned}")
    lines.append("P_{\\mathrm{EMG-07}} > P_{\\mathrm{EMG-03}} > P_{\\mathrm{EMG-05}} > P_{\\mathrm{EMG-06}} > P_{\\mathrm{EMG-04}} > P_{\\mathrm{EMG-02}} > P_{\\mathrm{EMG-01}}")
    lines.append("\\end{aligned}")
    lines.append("$$")
    lines.append("")
    lines.append("- **Priority Invariant:** Higher priority contingency triggers preempt lower priority states unconditionally.")
    lines.append("- **Deterministic Timing:** Maximum detection-to-actuation latency $t_{\\mathrm{resp}} \\le \\tau_{\\mathrm{deadline}}$ across all triggers.")
    lines.append("- **Fail-Safe Retention:** Non-reentrant emergency containment locks until authorized manual ground reset.")
    lines.append("")
    lines.append("### 12.2 Deterministic Emergency Statechart & State Machine")
    lines.append("```mermaid")
    lines.append("stateDiagram-v2")
    lines.append("    [*] --> Phase_Startup")
    lines.append("    Phase_Startup --> Phase_NominalExecution : BIT_Pass")
    lines.append("    Phase_NominalExecution --> Degraded_SensorFailsafe : EMG_04_SensorFault")
    lines.append("    Phase_NominalExecution --> Contingency_LostLinkReturn : EMG_01_LostC2")
    lines.append("    Phase_NominalExecution --> Contingency_DeadReckoning : EMG_02_GNSSLoss")
    lines.append("    Phase_NominalExecution --> Contingency_ResourceDivert : EMG_03_PowerDepletion")
    lines.append("    Phase_NominalExecution --> Contingency_GeofenceContainment : EMG_05_GeofenceBreach")
    lines.append("    Phase_NominalExecution --> Contingency_PrecautionaryLand : EMG_06_StructuralAnomaly")
    lines.append("    Phase_NominalExecution --> Emergency_FlightTermination : EMG_07_AbortCommand")
    lines.append("    Degraded_SensorFailsafe --> Contingency_LostLinkReturn : LinkTimeout")
    lines.append("    Contingency_LostLinkReturn --> Phase_SecureShutdown : SafeTouchdown")
    lines.append("    Contingency_DeadReckoning --> Phase_SecureShutdown : SafeTouchdown")
    lines.append("    Contingency_ResourceDivert --> Phase_SecureShutdown : SafeTouchdown")
    lines.append("    Contingency_GeofenceContainment --> Contingency_ResourceDivert : ContainmentHold")
    lines.append("    Contingency_PrecautionaryLand --> Phase_SecureShutdown : Touchdown")
    lines.append("    Emergency_FlightTermination --> Phase_SecureShutdown : ImpactSafe")
    lines.append("    Phase_SecureShutdown --> [*]")
    lines.append("```")
    lines.append("")
    lines.append("### 12.3 Degraded Modes & Fallback Hierarchy")
    lines.append("- **Tier 1 (Nominal Execution):** Full multi-sensor fusion, dual-channel C2 links, and nominal envelope margins.")
    lines.append("- **Tier 2 (Degraded Sensor Mode):** Single-sensor failure activates secondary observer and dead reckoning.")
    lines.append("- **Tier 3 (Contingency Link Mode):** Loss of primary C2 link triggers autonomous hold and return-to-base sequence.")
    lines.append("- **Tier 4 (Emergency Containment Mode):** Unrecoverable fault triggers ballistic parachute deploy or instant motor cutoff.")
    lines.append("")
    lines.append("### 12.4 Human-in-the-Loop (HITL) Authority & Override Protocols")
    lines.append("- **Supervisory Authority:** Operator retains positive manual override capability via independent emergency link.")
    lines.append("- **Dual-Consent Authentication:** Critical flight termination (`EMG-07`) requires two-operator verified consent keys.")
    lines.append("- **Interlock Inhibit:** Flight computer rejects manual commands that violate dynamic geofence containment limits.")
    lines.append("")
    lines.append("### 12.5 Autonomous Divert & Secondary Recovery Protocols")
    lines.append("- **Primary Recovery:** Designated nominal landing site or recovery zone.")
    lines.append("- **Secondary Divert Sites:** Pre-surveyed alternate recovery coordinates evaluated dynamically against Bingo energy.")
    lines.append("- **Terrain Clearance:** All emergency divert trajectories maintain minimum statutory altitude separation.")
    lines.append("")
    lines.append("### 12.6 Post-Emergency Containment, Latching & Reset Procedures")
    lines.append("- **Safety Lockout:** Emergency shutdown latches all actuators and high-voltage buses in de-energized safe states.")
    lines.append("- **Non-Volatile Blackbox Offload:** Diagnostic fault logs, sensor telemetry, and watchdog stack traces are securely written to non-volatile flash.")
    lines.append("- **Authorized Ground Clearance:** Physical inspection and signed maintenance clearance required before clearing failsafe lock.")
    lines.append("")

    # To guarantee lines >= 800, add comprehensive narrative notes and annex commentary
    for i in range(1, 70):
        lines.append(f"### 12.7.{i} Continuous Verification and Safety Assurance Note {i:02d}")
        lines.append(f"Normative safety assurance protocol {i:02d} enforces deterministic execution under all operational envelope deviations.")
        lines.append(f"Verification mechanism {i:02d} executes periodic sanity assessments, ensuring zero uncommanded state transitions.")
        lines.append(f"Audit log record {i:02d} is cryptographically sealed and transmitted across verified telemetry channels.")
        lines.append(f"Operational constraint rule {i:02d} validates cross-channel data integrity conforming to DO-178C DAL-A.")
        lines.append("")

    return "\n".join(lines)


def _get_valid_mission_intent_content() -> str:
    lines = []
    lines.append("| Attribute | Value |")
    lines.append("| :--- | :--- |")
    lines.append("| **Title** | Tactical Mission Intent & Execution Plan |")
    lines.append("| **Version** | 1.0.0 |")
    lines.append("| **Date** | 2026-09-01 |")
    lines.append("")
    lines.append("# Tactical Mission Intent & Execution Plan")
    lines.append("")
    lines.append("## 1. Commander's Intent & Operational Objectives")
    lines.append("")
    lines.append("### 1.1 Strategic Operational Purpose")
    lines.append("The tactical platform executes autonomous multi-point perimeter surveillance, situational telemetry broadcast, and dynamic boundary containment.")
    lines.append("The system operates in accordance with commander's intent to guarantee 100% mission task execution while maintaining statutory safety margins.")
    lines.append("")
    lines.append("### 1.2 Key Tactical Mission Tasks")
    lines.append("Tactical operations proceed through structured phases:")
    lines.append("1. **Pre-Mission Initialization & Autonomous Staging:** Confirmation of BIT diagnostics and perimeter boundary validation.")
    lines.append("2. **Corridor Ingress & Dynamic Trajectory Tracking:** Automated ingress following 4D waypoints with cross-track error $< 2.0\\text{ m}$.")
    lines.append("3. **Persistent Area Observation & Telemetry Streaming:** Multi-spectral observation, on-station orbit, and continuous telemetry relay.")
    lines.append("4. **Dynamic Deconfliction & Geo-Zone Enforcement:** Autonomous avoidance of keep-out zones and separation preservation.")
    lines.append("5. **Energy-Governed Return & Recovery:** Bingo energy monitoring ensuring statutory reserve $> 20\\%$ at touchdown.")
    lines.append("")
    lines.append("### 1.3 Desired Mission End State")
    lines.append("Successful mission termination is defined by:")
    lines.append("- 100% traversal and validation of designated mission waypoints.")
    lines.append("- Zero containment breaches across all operating phases.")
    lines.append("- Complete cryptographic audit log transfer to base station.")
    lines.append("- Touchdown at designated recovery pad with remaining energy $E_{\\mathrm{reserve}} \\ge 0.20 \\cdot E_{\\mathrm{capacity}}$.")
    lines.append("")
    lines.append("## 2. Mission Essential Task List (METL)")
    lines.append("")
    lines.append("### 2.1 Doctrinal METL Task Register")
    lines.append("The doctrinal METL roster defines mandatory capabilities with quantitative verification metrics:")
    lines.append("")
    lines.append("| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| `MET-01` | PreFlightSystemCheckout | Pre-launch power on and staging | 100% PBIT pass in < 30 s | Automated BIT Log | `/// OperationalAllocation: [MET-01]` |")
    lines.append("| `MET-02` | AutonomousIngressTransit | En-route nominal corridor | Cross-track error < 2.0 m | Flight Log Review | `/// OperationalAllocation: [MET-02]` |")
    lines.append("| `MET-03` | AreaSurveillanceOrbit | On-station target orbit | Orbit radius 100 m +/- 5 m | Telemetry Stream | `/// OperationalAllocation: [MET-03]` |")
    lines.append("| `MET-04` | AutonomousAreaSearch | Wide area search mode | 95% ground area coverage | Map Coverage Log | `/// OperationalAllocation: [MET-04]` |")
    lines.append("| `MET-05` | ActiveBoundaryDeconfliction | Dynamic boundary proximity | Zero perimeter violations | Geofence Alarm Log | `/// OperationalAllocation: [MET-05]` |")
    lines.append("| `MET-06` | MultiDomainThreatMitigation | Threat vector detection | Response time < 0.20 s | Safety Log Review | `/// OperationalAllocation: [MET-06]` |")
    lines.append("| `MET-07` | SafeReturnAndDivert | Return to recovery point | Touchdown error < 1.0 m | Visual / RTK Log | `/// OperationalAllocation: [MET-07]` |")
    lines.append("| `MET-08` | PostMissionDataOffload | Post-landing shutdown | Secure telemetry offload | Hash Verification | `/// OperationalAllocation: [MET-08]` |")
    lines.append("")
    lines.append("## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics")
    lines.append("")
    lines.append("### 3.1 INCOSE SEH v5.0 Quantitative Performance Metrics")
    lines.append("Formal KaTeX formulations define quantitative system effectiveness and performance thresholds:")
    lines.append("")
    lines.append("| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| MoE-01 | MoE | Mission Area Coverage Ratio | A_covered / A_total | 0.90 | 0.99 | Dimensionless |")
    lines.append("| MoE-02 | MoE | Operational Containment Integrity | 1.0 - (N_breaches / N_sorties) | 1.00 | 1.00 | Dimensionless |")
    lines.append("| MoE-03 | MoE | Mission Availability Factor | T_available / T_scheduled | 0.95 | 0.99 | Dimensionless |")
    lines.append("| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | 5.0 | 1.0 | m |")
    lines.append("| MoP-02 | MoP | Telemetry Latency Bound | tau_transport | 50.0 | 10.0 | ms |")
    lines.append("| MoP-03 | MoP | Emergency Containment Reaction Time | t_detect_to_actuate | 0.10 | 0.02 | s |")
    lines.append("| MoP-04 | MoP | Energy Consumption Rate | P_average | 450.0 | 350.0 | W |")
    lines.append("")
    lines.append("## 4. Threat & Electronic Warfare (EW) / Cyber Environment Matrix")
    lines.append("")
    lines.append("### 4.1 Multi-Domain Threat Matrix (10 Operational Domains)")
    lines.append("Threat vectors across all 10 canonical domains are analyzed with public clause citations:")
    lines.append("")
    lines.append("| Threat ID | Threat Domain | Threat Vector | Technical Description | Severity | Detection Mechanism | Autonomous Mitigation Rule | Public Clause Citation |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| `THR-KIN-01` | Kinetic | Ballistic projectile intercept | High-velocity physical impact trajectory | Critical | Optical/radar trajectory tracker | Execute evasive lateral displacement maneuver | MIL-STD-882E §4.3 |")
    lines.append("| `THR-MEC-01` | Mechanical | Actuator / motor jam | Motor bearing seizure or control surface lock | Critical | RPM feedback disparity & current spike | Differential thrust compensation and divert to base | MIL-STD-882E §4.3 |")
    lines.append("| `THR-PWR-01` | Power/Thermal | Battery cell thermal runaway | Internal cell short circuit causing thermal rise | Catastrophic | BMS thermistor array temperature trip | Disconnect faulted pack module and route to divert site | MIL-STD-882E §4.3 |")
    lines.append("| `THR-ENV-01` | Environmental | Severe convective wind gust | Sudden microburst exceeding flight stability envelope | High | Pitot-static air data velocity spike | Transition to high-stability penetration attitude | MIL-STD-810H Method 514.8 |")
    lines.append("| `THR-EWC-01` | EW | GNSS carrier jamming | C/N0 signal drop below tracking threshold | High | RAIM integrity alarm & signal-to-noise monitor | Revert to optical dead-reckoning and alternate PACE tier | STANAG 4586 §3.2 |")
    lines.append("| `THR-CYB-01` | Cyber | Telemetry packet injection | Unauthenticated frames on command uplink port | Critical | Cryptographic HMAC validation failure | Drop unauthorized frames, cycle crypto keys, isolate port | NIST SP 800-82r3 §5.2 |")
    lines.append("| `THR-OPT-01` | Optical | Directed laser blinding | High-intensity optical dazzle on vision tracker | High | Optical sensor pixel saturation detector | Deploy mechanical shutter filter and switch sensor modality | MIL-STD-882E §4.3 |")
    lines.append("| `THR-SIG-01` | Signature | Acoustic emission resonance | Airframe acoustic signature spike in quiet zone | Medium | Internal acoustic microphone / vibration transducer | Throttle back rotor RPM schedule to reduce observability | MIL-STD-882E §4.3 |")
    lines.append("| `THR-HUM-01` | Human Factors | Command input disparity | Conflicting pilot mode override command sequence | High | Command rate-of-change and state validity monitor | Enforce flight envelope protection and interlock rules | ISO/IEC/IEEE 29148 §6.4 |")
    lines.append("| `THR-CBRN-01` | CBRN | Corrosive aerosol plume | Ingress into toxic chemical atmospheric cloud | High | Optical particulate counter and chemical transducer | Seal internal avionics bay vents and initiate emergency divert | MIL-STD-810H Method 509.7 |")
    lines.append("")
    lines.append("## 5. PACE C2 Link Communications Plan")
    lines.append("")
    lines.append("### 5.1 Communications Plan Architecture")
    lines.append("The 4-tier PACE communications infrastructure ensures resilient connectivity across all operating regimes:")
    lines.append("")
    lines.append("| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Priority / Role |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| **Primary** | Point-to-Point COFDM | 5.8 GHz ISM | 10.0 Mbps | 2.0 s | Video & High-rate Telemetry |")
    lines.append("| **Alternate** | Cellular LTE / 5G VPN | Band 28 / 700 MHz | 2.0 Mbps | 3.0 s | Encrypted Cloud Relay |")
    lines.append("| **Contingency** | 900 MHz FHSS Radio | 915 MHz ISM | 115.2 kbps | 5.0 s | Essential C2 Commands Only |")
    lines.append("| **Emergency** | Satellite Iridium SBD | 1.6 GHz L-Band | 2.4 kbps | 10.0 s | Emergency Flight Termination & Geo-Beacon |")
    lines.append("")
    lines.append("## 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks")
    lines.append("")
    lines.append("### 6.1 Doctrinal Rules of Engagement")
    lines.append("- **ROE-01:** System shall not execute autonomous descent below $30\\text{ m}$ without positive ground radar clearance.")
    lines.append("- **ROE-02:** Optical sensor active tracking mode requires positive human-in-the-loop (HITL) authorization key.")
    lines.append("- **ROE-03:** Automated flight termination sequence requires two-person rule authenticated confirmation.")
    lines.append("- **ROE-04:** Dynamic exclusion zone ingress is prohibited under all operational modes.")
    lines.append("- **ROE-05:** Divert routing must preserve statutory minimum terrain clearance of $50.0\\text{ m}$.")
    lines.append("- **ROE-06:** Payload active emissions shall cease immediately upon lost-link detection.")
    lines.append("")
    lines.append("## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones")
    lines.append("")
    lines.append("### 7.1 Separation Minima & Dynamic Geo-Zones")
    lines.append("- **Primary Geofence Corridor:** Outer polygon bounding perimeter with $50\\text{ m}$ warning buffer.")
    lines.append("- **Keep-Out Zones (Dynamic):** Populated area buffer circles ($R = 300\\text{ m}$) marked NO-FLY.")
    lines.append("- **Separation Minima:** Maintain $150\\text{ m}$ vertical and $500\\text{ m}$ horizontal separation from non-cooperative targets.")
    lines.append("")
    lines.append("## 8. Go/No-Go Decision Matrix")
    lines.append("")
    lines.append("### 8.1 Operational Go/No-Go Criteria")
    lines.append("Deterministic decision logic governs mission progression across all checkpoints:")
    lines.append("")
    lines.append("| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
    lines.append("| GNG-01 | Pre-Launch | Battery State of Charge | >= 95.0% | Smart Battery BMS | Abort Launch if < 95% |")
    lines.append("| GNG-02 | Pre-Launch | Wind Velocity | <= 12.0 m/s | Ground Anemometer | Hold Launch if > 12 m/s |")
    lines.append("| GNG-03 | In-Flight | Navigation FOM | <= 2.0 | GPS Receiver HDOP | Initiate Loiter if > 2.0 |")
    lines.append("| GNG-04 | In-Flight | Motor Temperature | <= 85.0 degC | ESC Thermistor | Divert to Base if > 85 degC |")
    lines.append("| GNG-05 | In-Flight | C2 Link Packet Loss | <= 5.0% | Datalink RSSI Monitor | Switch to Alternate if > 5% |")
    lines.append("| GNG-06 | In-Flight | Geofence Distance Margin | >= 50.0 m | Boundary Estimator | Execute Turnaround if < 50 m |")
    lines.append("")
    lines.append("## 9. Bingo Energy Mathematics & Secondary Divert Protocols")
    lines.append("")
    lines.append("### 9.1 Mathematical Formulation of Bingo Energy Dynamics")
    lines.append("The Bingo Energy dynamics model calculates the real-time energy threshold $E_{\\mathrm{bingo}}(t)$ required for safe recovery while enforcing statutory reserve ratio $\\ge 20\\%$:")
    lines.append("")
    lines.append("$$")
    lines.append("\\begin{aligned}")
    lines.append("E_{\\mathrm{bingo}}(t) &= E_{\\mathrm{return}}(\\mathbf{p}(t), \\mathbf{p}_{\\mathrm{dest}}) + E_{\\mathrm{divert}}(\\mathbf{p}_{\\mathrm{dest}}, \\mathbf{p}_{\\mathrm{alt}}) + E_{\\mathrm{reserve}} + E_{\\mathrm{contingency}} \\\\")
    lines.append("E_{\\mathrm{reserve}} &\\ge 0.20 \\cdot E_{\\mathrm{capacity}}")
    lines.append("\\end{aligned}")
    lines.append("$$")
    lines.append("")
    lines.append("### 9.2 Energy Parameter Table")
    lines.append("The nominal stored energy budget parameters are defined in the following table:")
    lines.append("")
    lines.append("| Energy Parameter | Symbol | Value | Units | Constraint Rule |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| Total Pack Capacity | E_capacity | 500.0 | kJ | Total nominal battery stored energy |")
    lines.append("| Return Transit Energy | E_return | 150.0 | kJ | Energy required for primary return trajectory |")
    lines.append("| Secondary Divert Energy | E_divert | 60.0 | kJ | Energy required to divert to secondary recovery site |")
    lines.append("| Mandatory Statutory Reserve | E_reserve | 100.0 | kJ | 20.0% statutory reserve threshold (E_reserve >= 0.20 * E_capacity) |")
    lines.append("| Contingency Buffer | E_contingency | 40.0 | kJ | Holding pattern & go-around reserve |")
    lines.append("| Total Bingo Threshold | E_bingo | 350.0 | kJ | Critical return threshold (E_current <= E_bingo -> Divert) |")
    lines.append("")
    lines.append("## 10. Gate 24 MissionTask Traceability Tags (Allocation Tags)")
    lines.append("- `/// OperationalAllocation: [MET-01]`")
    lines.append("- `/// OperationalAllocation: [MET-02]`")
    lines.append("- `/// OperationalAllocation: [MET-03]`")
    lines.append("- `/// OperationalAllocation: [MET-04]`")
    lines.append("- `/// OperationalAllocation: [MET-05]`")
    lines.append("- `/// OperationalAllocation: [MET-06]`")
    lines.append("- `/// OperationalAllocation: [MET-07]`")
    lines.append("- `/// OperationalAllocation: [MET-08]`")
    lines.append("")
    
    # Ensure lines >= 400
    for i in range(1, 55):
        lines.append(f"### 10.1.{i} Tactical Execution Task Rule {i:02d}")
        lines.append(f"Tactical mission execution rule {i:02d} provides verified determinism across all multi-domain operating conditions.")
        lines.append(f"Verification mechanism {i:02d} ensures continuous compliance with Gate 24 traceability and commander intent.")
        lines.append(f"Safety assurance rule {i:02d} validates cross-channel communication integrity.")
        lines.append("")

    return "\n".join(lines)


class TestConOpsAndMissionIntentValidators(unittest.TestCase):
    """
    Comprehensive test suite for Gate 26: ConOps & Mission Intent Completeness Validators.
    """

    def test_clean_upstream_landing_zones_pass_gracefully(self):
        """Clean upstream workspace with empty landing zone returns zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "upstream"), exist_ok=True)
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            self.assertEqual(conops_val.validate(repo), [])
            self.assertEqual(mission_val.validate(repo), [])

    def test_downstream_missing_conops_corpus_fails(self):
        """Downstream workspace missing docs/conops returns fail-closed corpus-missing findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Downstream workspace: no .pipeline/upstream marker
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            conops_findings = conops_val.validate(repo)
            mission_findings = mission_val.validate(repo)

            self.assertEqual(len(conops_findings), 1)
            self.assertEqual(conops_findings[0].rule_id, "conops-corpus-missing")
            self.assertEqual(conops_findings[0].location, "docs/conops")

            self.assertEqual(len(mission_findings), 1)
            self.assertEqual(mission_findings[0].rule_id, "mission-intent-corpus-missing")
            self.assertEqual(mission_findings[0].location, "docs/conops")

    def test_downstream_empty_conops_dir_fails(self):
        """Downstream workspace with empty docs/conops returns fail-closed corpus-missing findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            conops_findings = conops_val.validate(repo)
            mission_findings = mission_val.validate(repo)

            self.assertEqual(len(conops_findings), 1)
            self.assertEqual(conops_findings[0].rule_id, "conops-corpus-missing")

            self.assertEqual(len(mission_findings), 1)
            self.assertEqual(mission_findings[0].rule_id, "mission-intent-corpus-missing")

    def test_mock_downstream_missing_conops_dir_emits_findings(self):
        """Mock WorkspaceRepository in downstream mode emits findings when docs/conops is missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            mock_repo = MagicMock(spec=WorkspaceRepository)
            mock_repo.workspace_dir = tmpdir
            mock_repo.is_upstream_compiler_repo.return_value = False

            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            conops_findings = conops_val.validate(mock_repo)
            mission_findings = mission_val.validate(mock_repo)

            self.assertEqual(len(conops_findings), 1)
            self.assertEqual(conops_findings[0].rule_id, "conops-corpus-missing")
            self.assertEqual(conops_findings[0].location, "docs/conops")

            self.assertEqual(len(mission_findings), 1)
            self.assertEqual(mission_findings[0].rule_id, "mission-intent-corpus-missing")
            self.assertEqual(mission_findings[0].location, "docs/conops")

    def test_mock_downstream_missing_canonical_documents_emits_findings(self):
        """Mock WorkspaceRepository in downstream mode emits findings when CONOPS.md or MISSION_INTENT.md is missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            mock_repo = MagicMock(spec=WorkspaceRepository)
            mock_repo.workspace_dir = tmpdir
            mock_repo.is_upstream_compiler_repo.return_value = False

            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            # Case 1: Empty conops directory
            self.assertEqual(len(conops_val.validate(mock_repo)), 1)
            self.assertEqual(conops_val.validate(mock_repo)[0].rule_id, "conops-corpus-missing")
            self.assertEqual(len(mission_val.validate(mock_repo)), 1)
            self.assertEqual(mission_val.validate(mock_repo)[0].rule_id, "mission-intent-corpus-missing")

            # Case 2: Only valid MISSION_INTENT.md present (CONOPS.md missing)
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

            conops_findings = conops_val.validate(mock_repo)
            mission_findings = mission_val.validate(mock_repo)
            self.assertEqual(len(conops_findings), 1)
            self.assertEqual(conops_findings[0].rule_id, "conops-corpus-missing")
            self.assertEqual(mission_findings, [])

            # Case 3: Only valid CONOPS.md present (MISSION_INTENT.md missing)
            os.remove(os.path.join(conops_dir, "MISSION_INTENT.md"))
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            conops_findings = conops_val.validate(mock_repo)
            mission_findings = mission_val.validate(mock_repo)
            self.assertEqual(conops_findings, [])
            self.assertEqual(len(mission_findings), 1)
            self.assertEqual(mission_findings[0].rule_id, "mission-intent-corpus-missing")

            # Case 4: Only template files present
            os.remove(os.path.join(conops_dir, "CONOPS.md"))
            with open(os.path.join(conops_dir, "CONOPS_CANONICAL_TEMPLATE.md"), "w", encoding="utf-8") as f:
                f.write("# Template")
            with open(os.path.join(conops_dir, "MISSION_INTENT_CANONICAL_TEMPLATE.md"), "w", encoding="utf-8") as f:
                f.write("# Template")

            self.assertEqual(len(conops_val.validate(mock_repo)), 1)
            self.assertEqual(conops_val.validate(mock_repo)[0].rule_id, "conops-corpus-missing")
            self.assertEqual(len(mission_val.validate(mock_repo)), 1)
            self.assertEqual(mission_val.validate(mock_repo)[0].rule_id, "mission-intent-corpus-missing")

    def test_mock_upstream_missing_conops_corpus_and_documents_return_empty(self):
        """Mock WorkspaceRepository in upstream mode returns [] when docs/conops or canonical documents are missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            mock_repo = MagicMock(spec=WorkspaceRepository)
            mock_repo.workspace_dir = tmpdir
            mock_repo.is_upstream_compiler_repo.return_value = True

            conops_val = ConopsCompletenessValidator()
            mission_val = MissionIntentCompletenessValidator()

            # Case 1: Missing docs/conops directory
            self.assertEqual(conops_val.validate(mock_repo), [])
            self.assertEqual(mission_val.validate(mock_repo), [])

            # Case 2: Empty docs/conops directory
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            self.assertEqual(conops_val.validate(mock_repo), [])
            self.assertEqual(mission_val.validate(mock_repo), [])

            # Case 3: Only CONOPS.md present
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())
            self.assertEqual(conops_val.validate(mock_repo), [])
            self.assertEqual(mission_val.validate(mock_repo), [])

            # Case 4: Only MISSION_INTENT.md present
            os.remove(os.path.join(conops_dir, "CONOPS.md"))
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())
            self.assertEqual(conops_val.validate(mock_repo), [])
            self.assertEqual(mission_val.validate(mock_repo), [])

            # Case 5: Only templates present
            os.remove(os.path.join(conops_dir, "MISSION_INTENT.md"))
            with open(os.path.join(conops_dir, "CONOPS_CANONICAL_TEMPLATE.md"), "w", encoding="utf-8") as f:
                f.write("# Template")
            self.assertEqual(conops_val.validate(mock_repo), [])
            self.assertEqual(mission_val.validate(mock_repo), [])

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
            content = content.replace("200.0 | m | Declared ground risk buffer containment radius", "20.0 | m | Declared ground risk buffer containment radius")

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

    def test_mission_intent_metl_task_prose_suffixes_extract_digits_only(self):
        """METL tasks with prose suffixes (e.g. MET-01-OperationalPayload, MET-02-FlightGuidance) extract digits-only IDs (MET-01, MET-02) satisfied by standard allocation tags."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Replace MET-01, MET-02, MET-03 in table with suffixed prose
            content = content.replace("`MET-01`", "`MET-01-OperationalPayload`")
            content = content.replace("`MET-02`", "`MET-02-FlightGuidance`")
            content = content.replace("`MET-03`", "`MET-3-PayloadTelepresence`")

            # Add bullet points with hyphens and prose in Section 2
            content = content.replace(
                "## 2. Mission Essential Task List (METL)",
                "## 2. Mission Essential Task List (METL)\n\n"
                "- **MET-01-OperationalPayload**: System initial payload activation.\n"
                "- **MET-02-FlightGuidance**: En-route corridor navigation.\n"
                "- **MET-3-PayloadTelepresence**: Real-time downlink telemetry.\n"
                "- Note: {{MET_01_TASK_NAME}} placeholder token should not be parsed as task ID.\n"
                "- Trailing words: MET-02-FlightGuidance-nominal-profile should not swallow words.\n"
            )

            # Allocation tags in Section 10 remain standard digits-only tags:
            # /// OperationalAllocation: [MET-01]
            # /// OperationalAllocation: [MET-02]
            # /// OperationalAllocation: [MET-03]
            # /// OperationalAllocation: [MET-04]
            # /// OperationalAllocation: [MET-05]
            # /// OperationalAllocation: [MET-06]

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            alloc_errors = [f for f in findings if f.rule_id == "mission-metl-unallocated"]
            self.assertEqual(alloc_errors, [], f"Unexpected unallocated METL findings: {alloc_errors}")

    def test_mission_intent_metl_task_prose_suffix_unallocated_fails_with_digits_only_id(self):
        """Unallocated METL task declared with prose suffix (MET-01-OperationalPayload) reports failure with digits-only ID ('MET-01')."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Replace MET-01 with MET-01-OperationalPayload in table
            content = content.replace("`MET-01`", "`MET-01-OperationalPayload`")
            # Strip allocation tag for MET-01 from Section 2 and Section 10
            content = content.replace("`/// OperationalAllocation: [MET-01]`", "—")
            content = content.replace("- `/// OperationalAllocation: [MET-01]`\n", "")

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            alloc_errors = [f for f in findings if f.rule_id == "mission-metl-unallocated"]
            self.assertTrue(len(alloc_errors) >= 1)
            # Must strictly be digits-only normalized ID 'MET-01'
            self.assertEqual(alloc_errors[0].detail.get("task_id"), "MET-01")
            self.assertIn("MET-01", str(alloc_errors[0]))
            self.assertNotIn("OPERATIONALPAYLOAD", str(alloc_errors[0]))

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

            # Assert subsections 12.1 through 12.6 and statechart presence in synthesized ConOps
            for sub_idx in range(1, 7):
                self.assertIn(f"### 12.{sub_idx}", c_txt)
            self.assertIn("stateDiagram-v2", c_txt)

            # Assert 10 threat domains presence in synthesized Mission Intent
            threat_domains = [
                "Kinetic", "Mechanical", "Power/Thermal", "Environmental",
                "EW", "Cyber", "Optical", "Signature", "Human Factors", "CBRN"
            ]
            for domain in threat_domains:
                self.assertIn(domain, m_txt)

    def test_conops_missing_emergency_depth_subsections_fails(self):
        """ConOps missing subsections 12.3..12.6 fails with conops-emergency-depth-missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Truncate after 12.2 (removing 12.3..12.6)
            idx_12_3 = content.find("### 12.3")
            self.assertNotEqual(idx_12_3, -1)
            content_truncated = content[:idx_12_3]

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content_truncated)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            depth_errors = [f for f in findings if f.rule_id == "conops-emergency-depth-missing"]
            self.assertEqual(len(depth_errors), 1)
            missing = depth_errors[0].detail.get("missing_subsections", [])
            self.assertTrue(any("12.3" in m for m in missing))
            self.assertTrue(any("12.4" in m for m in missing))
            self.assertTrue(any("12.5" in m for m in missing))
            self.assertTrue(any("12.6" in m for m in missing))

    def test_conops_missing_emergency_statechart_fails(self):
        """ConOps missing Mermaid statechart in Section 12 fails with conops-emergency-statechart-missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_conops_content()
            # Remove Mermaid stateDiagram-v2 block
            content_no_chart = re.sub(r'```mermaid[\s\S]*?```', '', content)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content_no_chart)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            chart_errors = [f for f in findings if f.rule_id == "conops-emergency-statechart-missing"]
            self.assertEqual(len(chart_errors), 1)

    def test_conops_complete_emergency_depth_and_statechart_passes(self):
        """ConOps with complete Section 12 depth and Mermaid statechart passes Gate 26 validation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            self.assertEqual(findings, [])

    def test_mission_intent_missing_threat_domains_fails(self):
        """Mission Intent missing required threat domains fails with mission-threat-domain-missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            content = _get_valid_mission_intent_content()
            # Replace 10-domain threat matrix with only EW and Cyber rows
            sparse_sec4 = """## 4. Threat & Electronic Warfare (EW) / Cyber Environment Matrix
| Threat ID | Threat Domain | Threat Vector | Technical Description | Severity | Detection Mechanism | Autonomous Mitigation Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `THR-EWC-01` | EW | GNSS carrier jamming | C/N0 drop | High | RAIM alert | Revert to inertial dead-reckoning | STANAG 4586 §3.2 |
| `THR-CYB-01` | Cyber | Telemetry packet injection | Unauthenticated frames | Critical | HMAC failure | Drop unauthorized frames | NIST SP 800-82r3 §5.2 |
"""
            content_sparse = re.sub(r'## 4\. Threat[\s\S]*?(?=## 5\.)', sparse_sec4, content)

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content_sparse)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            threat_errors = [f for f in findings if f.rule_id == "mission-threat-domain-missing"]
            self.assertEqual(len(threat_errors), 1)
            missing = threat_errors[0].detail.get("missing_domains", [])
            self.assertIn("Kinetic", missing)
            self.assertIn("Mechanical", missing)
            self.assertIn("Power/Thermal", missing)
            self.assertIn("Environmental", missing)
            self.assertIn("Optical", missing)
            self.assertIn("Signature", missing)
            self.assertIn("Human Factors", missing)
            self.assertIn("CBRN", missing)

    def test_mission_intent_complete_10_domain_threat_matrix_passes(self):
        """Mission Intent with all 10 threat domains passes Gate 26 threat matrix density validation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            self.assertEqual(findings, [])

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

    def test_conops_missing_allocated_obligation_fails(self):
        """When RESEARCH_INVENTORY.md allocates an obligation to CONOPS.md, missing witness tag emits conops-obligation-unwitnessed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            research_dir = os.path.join(tmpdir, "docs", "research")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(research_dir, exist_ok=True)

            inventory_content = """# Research Inventory
## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO | RE | §6.4.2 | Requirements | 1 | §6.4.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Requirements | ISO/IEC/IEEE 29148:2018 | 1 | Inspection | §6.4.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | ISO/IEC/IEEE 29148:2018 | §6.4.2 | ConOps Req | Phase 1 (Structural) | `docs/conops/CONOPS.md` |
"""
            with open(os.path.join(research_dir, "RESEARCH_INVENTORY.md"), "w", encoding="utf-8") as f:
                f.write(inventory_content)

            # CONOPS.md has all 12 sections but NO tag for OBL-01
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            unwitnessed = [f for f in findings if f.rule_id == "conops-obligation-unwitnessed"]
            self.assertEqual(len(unwitnessed), 1)
            self.assertEqual(unwitnessed[0].detail.get("obligation_id"), "OBL-01")

    def test_conops_with_witnessed_allocated_obligation_passes(self):
        """When CONOPS.md includes witness tag for allocated obligation, conops-obligation-unwitnessed is not emitted."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            research_dir = os.path.join(tmpdir, "docs", "research")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(research_dir, exist_ok=True)

            inventory_content = """# Research Inventory
## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO | RE | §6.4.2 | Requirements | 1 | §6.4.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Requirements | ISO/IEC/IEEE 29148:2018 | 1 | Inspection | §6.4.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | ISO/IEC/IEEE 29148:2018 | §6.4.2 | ConOps Req | Phase 1 (Structural) | `docs/conops/CONOPS.md` |
"""
            with open(os.path.join(research_dir, "RESEARCH_INVENTORY.md"), "w", encoding="utf-8") as f:
                f.write(inventory_content)

            # CONOPS.md has witness tag for OBL-01
            conops_txt = _get_valid_conops_content() + "\n/// Realises: [OBL-01]\n"
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_txt)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            conops_val = ConopsCompletenessValidator()
            findings = conops_val.validate(repo)

            unwitnessed = [f for f in findings if f.rule_id == "conops-obligation-unwitnessed"]
            self.assertEqual(len(unwitnessed), 0)

    def test_mission_intent_missing_allocated_obligation_fails(self):
        """When RESEARCH_INVENTORY.md allocates an obligation to MISSION_INTENT.md, missing witness tag emits mission-intent-obligation-unwitnessed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            research_dir = os.path.join(tmpdir, "docs", "research")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(research_dir, exist_ok=True)

            inventory_content = """# Research Inventory
## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| NATO STANAG 4586 | NATO | UCS | §3.2 | Interoperability | 1 | §3.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INT-01` | Interoperability | NATO STANAG 4586 | 1 | Test | §3.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INT-01` | NATO STANAG 4586 | §3.2 | DLI Interface | Phase 1 (Structural) | `docs/conops/MISSION_INTENT.md` |
"""
            with open(os.path.join(research_dir, "RESEARCH_INVENTORY.md"), "w", encoding="utf-8") as f:
                f.write(inventory_content)

            # MISSION_INTENT.md has all 10 sections but NO tag for INT-01
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            unwitnessed = [f for f in findings if f.rule_id == "mission-intent-obligation-unwitnessed"]
            self.assertEqual(len(unwitnessed), 1)
            self.assertEqual(unwitnessed[0].detail.get("obligation_id"), "INT-01")

    def test_mission_intent_with_witnessed_allocated_obligation_passes(self):
        """When MISSION_INTENT.md includes witness tag for allocated obligation, mission-intent-obligation-unwitnessed is not emitted."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            research_dir = os.path.join(tmpdir, "docs", "research")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(research_dir, exist_ok=True)

            inventory_content = """# Research Inventory
## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| NATO STANAG 4586 | NATO | UCS | §3.2 | Interoperability | 1 | §3.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INT-01` | Interoperability | NATO STANAG 4586 | 1 | Test | §3.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INT-01` | NATO STANAG 4586 | §3.2 | DLI Interface | Phase 1 (Structural) | `docs/conops/MISSION_INTENT.md` |
"""
            with open(os.path.join(research_dir, "RESEARCH_INVENTORY.md"), "w", encoding="utf-8") as f:
                f.write(inventory_content)

            # MISSION_INTENT.md has witness tag for INT-01
            mission_txt = _get_valid_mission_intent_content() + "\n/// ObligationWitness: [INT-01]\n"
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(mission_txt)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            mission_val = MissionIntentCompletenessValidator()
            findings = mission_val.validate(repo)

            unwitnessed = [f for f in findings if f.rule_id == "mission-intent-obligation-unwitnessed"]
            self.assertEqual(len(unwitnessed), 0)

    def test_cli_integration_fails_when_conops_obligation_unwitnessed(self):
        """Verify CLI fails closed (exit code 1) when an allocated ConOps obligation is unwitnessed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            research_dir = os.path.join(tmpdir, "docs", "research")
            schema_dir = os.path.join(tmpdir, "schema")
            pipeline_dir = os.path.join(tmpdir, ".pipeline", "logical-ui")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(research_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(pipeline_dir, exist_ok=True)

            inventory_content = """# Research Inventory
## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO | RE | §6.4.2 | Requirements | 1 | §6.4.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Requirements | ISO/IEC/IEEE 29148:2018 | 1 | Inspection | §6.4.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | ISO/IEC/IEEE 29148:2018 | §6.4.2 | ConOps Req | Phase 1 (Structural) | `docs/conops/CONOPS.md` |
"""
            with open(os.path.join(research_dir, "RESEARCH_INVENTORY.md"), "w", encoding="utf-8") as f:
                f.write(inventory_content)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_conops_content())

            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(_get_valid_mission_intent_content())

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

            cli_py = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "cli.py")
            import subprocess
            res = subprocess.run(
                [sys.executable, cli_py, "--workspace", tmpdir, "--schema-only"],
                capture_output=True,
                text=True
            )
            self.assertEqual(res.returncode, 1, f"Expected CLI failure, but got exit code {res.returncode}:\n{res.stdout}")
            self.assertTrue(
                "conops-obligation-unwitnessed" in res.stdout
                or "coverage-digest-obligation-unrealized" in res.stdout
                or "obligation-unwitnessed" in res.stdout
                or "ConOps Completeness Violations Identified" in res.stdout
            )

    def test_conops_density_insufficient_fails(self):
        """ConOps specification with fewer than 800 lines emits conops-density-insufficient finding (Fixes #130)."""
        full_content = _get_valid_conops_content()
        short_content = "\n".join(full_content.splitlines()[:500])
        val = ConopsCompletenessValidator()
        findings = val._validate_conops_text(short_content, "docs/conops/CONOPS.md")
        density_findings = [f for f in findings if f.rule_id == "conops-density-insufficient"]
        self.assertEqual(len(density_findings), 1)
        self.assertIn("minimum required: 800 lines", str(density_findings[0]))

    def test_mission_density_insufficient_fails(self):
        """Mission Intent specification with fewer than 400 lines emits mission-density-insufficient finding (Fixes #130)."""
        full_content = _get_valid_mission_intent_content()
        short_content = "\n".join(full_content.splitlines()[:250])
        val = MissionIntentCompletenessValidator()
        findings = val._validate_mission_text(short_content, "docs/conops/MISSION_INTENT.md")
        density_findings = [f for f in findings if f.rule_id == "mission-density-insufficient"]
        self.assertEqual(len(density_findings), 1)
        self.assertIn("minimum required: 400 lines", str(density_findings[0]))

    def test_conops_tables_insufficient_fails(self):
        """ConOps specification with fewer than 8 markdown tables emits conops-tables-insufficient finding (Fixes #130)."""
        full_content = _get_valid_conops_content()
        # Filter out table rows while keeping line count >= 800
        lines = full_content.splitlines()
        non_table_lines = [l for l in lines if not (l.startswith("|") and l.endswith("|"))]
        while len(non_table_lines) < 850:
            non_table_lines.append("Additional descriptive line for testing non-table density floor.")
        content = "\n".join(non_table_lines)
        val = ConopsCompletenessValidator()
        findings = val._validate_conops_text(content, "docs/conops/CONOPS.md")
        table_findings = [f for f in findings if f.rule_id == "conops-tables-insufficient"]
        self.assertEqual(len(table_findings), 1)
        self.assertIn("minimum required: 8 formal tables", str(table_findings[0]))

    def test_conops_mermaid_diagrams_insufficient_fails(self):
        """ConOps specification missing required Mermaid diagram types emits conops-mermaid-diagrams-insufficient (Fixes #130)."""
        full_content = _get_valid_conops_content()
        # Remove flowchart TB
        broken_content = full_content.replace("flowchart TB", "graph LR")
        val = ConopsCompletenessValidator()
        findings = val._validate_conops_text(broken_content, "docs/conops/CONOPS.md")
        diagram_findings = [f for f in findings if f.rule_id == "conops-mermaid-diagrams-insufficient"]
        self.assertEqual(len(diagram_findings), 1)
        self.assertIn("flowchart TB", str(diagram_findings[0]))

    def test_conops_missing_pugh_matrix_fails(self):
        """ConOps specification missing Section 4 Pugh decision matrix emits conops-pugh-matrix-missing (Fixes #130)."""
        full_content = _get_valid_conops_content()
        # Remove Pugh decision matrix table and keyword in Section 4
        broken_content = re.sub(r'\| Evaluation Criterion \(i\)[\s\S]*?\n\n', '\n\n', full_content)
        broken_content = broken_content.replace("Pugh", "Trade-Off")
        val = ConopsCompletenessValidator()
        findings = val._validate_conops_text(broken_content, "docs/conops/CONOPS.md")
        pugh_findings = [f for f in findings if f.rule_id == "conops-pugh-matrix-missing"]
        self.assertEqual(len(pugh_findings), 1)
        self.assertIn("missing mandatory Pugh decision matrix", str(pugh_findings[0]))

    def test_conops_missing_pugh_sensitivity_equation_fails(self):
        """ConOps specification missing LaTeX sensitivity equation S_j(w) in Section 4 emits conops-pugh-sensitivity-missing (Fixes #130)."""
        full_content = _get_valid_conops_content()
        # Remove math block with S_j(w)
        broken_content = re.sub(r'\$\$\s*\\begin\{aligned\}\s*S_j\(w\)[\s\S]*?\\end\{aligned\}\s*\$\$', '$$ J = w_i $$', full_content)
        broken_content = broken_content.replace("S_j(w)", "Score")
        val = ConopsCompletenessValidator()
        findings = val._validate_conops_text(broken_content, "docs/conops/CONOPS.md")
        sens_findings = [f for f in findings if f.rule_id == "conops-pugh-sensitivity-missing"]
        self.assertEqual(len(sens_findings), 1)
        self.assertIn("missing mandatory LaTeX sensitivity equation S_j(w)", str(sens_findings[0]))

    def test_mission_intent_table_aware_energy_reserve_extraction_prevents_false_inequality_trigger(self):
        """
        Verify that mathematical inequality formulas in Section 9 (e.g. E_reserve >= 0.20 * E_capacity)
        do not cause false trigger on the 0.20 coefficient when table defines nominal capacity and reserve.
        Reference Fixes #130.
        """
        full_content = _get_valid_mission_intent_content()
        val = MissionIntentCompletenessValidator(strict_bingo_math=True)
        findings = val._validate_mission_text(full_content, "docs/conops/MISSION_INTENT.md")
        bingo_findings = [f for f in findings if f.rule_id == "mission-bingo-reserve-insufficient"]
        self.assertEqual(len(bingo_findings), 0, f"Unexpected bingo reserve findings: {bingo_findings}")

    def test_domain_agnosticism_no_uas_or_domain_specific_strings(self):
        """
        Verify that ConOps and Mission Intent completeness validators and canonical templates
        are 100% domain-agnostic without hardcoded UAS or domain-specific assumptions.
        Reference Fixes #130.
        """
        val_path = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "validators", "conops_completeness_validator.py")
        with open(val_path, "r", encoding="utf-8") as f:
            code = f.read()

        forbidden_terms = ["drone", "quadcopter", "multirotor", "fixed-wing", "vtol"]
        for term in forbidden_terms:
            self.assertNotIn(term, code.lower(), f"Validator code should be domain-agnostic; found '{term}'.")


if __name__ == "__main__":
    unittest.main()

