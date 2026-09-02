| Attribute | Value |
| :--- | :--- |
| **Title** | OMG UAF Operational Activity Taxonomy |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 6. OMG UAF Operational Activity Taxonomy

### 6.1 Operational Activity Decomposition
In accordance with OMG Unified Architecture Framework (UAF) v1.2 / v2.0 Operational Processes (Op-Pr) and Operational Taxonomy (Op-Tx), the system operational activities are formally decomposed and allocated 100% to system performer nodes, logical subsystems, and physical resources.

Every operational activity is bound to a machine-verifiable Gate 24 allocation tag (`/// OperationalAllocation: [OA-XX]`) and realization tag (`/// Realises: [UAF-ACT-XX]`).

| Activity ID | Activity Name | Allocated Performer Node & Resource | Operational Description | Traceability & Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- |
| **OA-01** | `SystemInitializationAndPBIT` | Flight Avionics Core / Secure MPU | Executes power-on Built-In-Tests (PBIT), cross-checks redundant sensor biases, validates actuator end-stops, verifies cryptographic root of trust, and logs calibration status within t_PBIT <= tau_PBIT_max. | `/// OperationalAllocation: [OA-01]`<br>`/// Realises: [UAF-ACT-01]` |
| **OA-02** | `AutonomousFlightGuidance` | Autopilot Subsystem / DO-178C DAL-A FCC | Computes and tracks 4D curved waypoint trajectories, executes closed-loop aerodynamic control at loop rate f_control, and performs cross-track error minimization (e_xtrack <= epsilon_xtrack_max). | `/// OperationalAllocation: [OA-02]`<br>`/// Realises: [UAF-ACT-02]` |
| **OA-03** | `PayloadDataAcquisition` | Sensor Gimbal Node / Edge Neural Compute | Controls multi-axis stabilized optical/thermal sensor payload, captures multi-spectral imagery, executes edge object detection, and formats geo-referenced video metadata. | `/// OperationalAllocation: [OA-03]`<br>`/// Realises: [UAF-ACT-03]` |
| **OA-04** | `ContinuousHealthMonitoring` | Safety Watchdog Subsystem / FPGA Fabric | Performs high-frequency cross-channel consistency checks at rate f_health_monitor across satellite navigation, redundant IMUs, barometric pressure, motor current draw, and energy state-of-charge. | `/// OperationalAllocation: [OA-04]`<br>`/// Realises: [UAF-ACT-04]` |
| **OA-05** | `DynamicAirspaceContainment` | Geofence Engine / UTM Client Subsystem | Evaluates 4D spatial boundary proximity against hard geofence limits and U-space dynamic keep-out zones at rate f_geofence; enforces automated maximum-rate turnaround upon warning threshold breach. | `/// OperationalAllocation: [OA-05]`<br>`/// Realises: [UAF-ACT-05]` |
| **OA-06** | `AutonomousContingencyManagement` | Deterministic Failsafe State Machine | Detects emergency triggers (`EMG-01`..`EMG-07`), arbitrates priorities, and executes deterministic containment maneuvers (RTH, secondary divert, or emergency termination) within t_resp <= tau_containment. | `/// OperationalAllocation: [OA-06]`<br>`/// Realises: [UAF-ACT-06]` |
| **OA-07** | `SecureDatalinkRelay` | PACE Communications Transceiver Node | Manages multi-tier encryption, monitors link quality (SNR, BER), executes rapid failover across primary, alternate, contingency, and emergency PACE datalinks within t_switch <= tau_switch_max. | `/// OperationalAllocation: [OA-07]`<br>`/// Realises: [UAF-ACT-07]` |
| **OA-08** | `PrecisionRecoveryAndShutdown` | Guidance Subsystem / Recovery Sensor Node | Executes automated precision approach using optical fiducial and RTK landing guidance, terminal flare, motor cutoff, mechanical brake lock, and post-flight cryptographic data zeroization. | `/// OperationalAllocation: [OA-08]`<br>`/// Realises: [UAF-ACT-08]` |

### 6.2 Performer Node Allocation Matrix
The following matrix demonstrates 100% allocation coverage across system nodes:

| System Performer Node | Node Type | Primary Resource | Allocated Activities |
| :--- | :--- | :--- | :--- |
| Air Vehicle Avionics Node | Cyber-Physical Air | Redundant Safety-Critical FCC | OA-01, OA-02, OA-04 |
| Multi-Spectral Payload Node | Sensor Edge Node | Edge Vision AI SoC | OA-03 |
| Safety Watchdog Node | Critical Hardware | Independent Hardware Watchdog / FPGA | OA-04, OA-05, OA-06 |
| PACE Transceiver Node | Datalink Router | Multi-Band RF Modem & Transceiver | OA-07 |
| Ground Control Station Node | Ground Performer | Ruggedized Operator Terminal | OA-01, OA-05, OA-07, OA-08 |

### 6.3 Operational Traceability Invariant
Per DEAP Governance Rule `rules/conops-mission-intent-integrity.md`, all downstream SysML v2 architectural blocks, Stateflow behavioral charts, and DO-178C software requirement units must carry direct traceability back to these eight canonical operational activities using the formalized allocation syntax:
- `/// OperationalAllocation: [OA-01]`
- `/// OperationalAllocation: [OA-02]`
- `/// OperationalAllocation: [OA-03]`
- `/// OperationalAllocation: [OA-04]`
- `/// OperationalAllocation: [OA-05]`
- `/// OperationalAllocation: [OA-06]`
- `/// OperationalAllocation: [OA-07]`
- `/// OperationalAllocation: [OA-08]`
