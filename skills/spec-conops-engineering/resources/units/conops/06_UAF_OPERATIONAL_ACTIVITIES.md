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
| **OA-01** | `SystemInitializationAndPBIT` | Flight Avionics Core / ARM Cortex-R5 MPU | Executes power-on Built-In-Tests (PBIT), cross-checks triple IMU biases, validates actuator end-stops, verifies cryptographic root of trust, and logs calibration status in < 30 s. | `/// OperationalAllocation: [OA-01]`<br>`/// Realises: [UAF-ACT-01]` |
| **OA-02** | `AutonomousFlightGuidance` | Autopilot Subsystem / DO-178C DAL-A FCC | Computes and tracks 4D curved waypoint trajectories, executes closed-loop aerodynamic control at 400 Hz, and performs cross-track error minimization (< 1.0 m). | `/// OperationalAllocation: [OA-02]`<br>`/// Realises: [UAF-ACT-02]` |
| **OA-03** | `PayloadDataAcquisition` | Sensor Gimbal Node / Edge Neural Compute | Controls dual-axis gyro-stabilized optical/thermal gimbal, captures 4K EO and 640x512 IR imagery, executes edge object detection, and formats geo-referenced video metadata. | `/// OperationalAllocation: [OA-03]`<br>`/// Realises: [UAF-ACT-03]` |
| **OA-04** | `ContinuousHealthMonitoring` | Safety Watchdog Subsystem / FPGA Fabric | Performs 500 Hz cross-channel consistency checks across dual GNSS, triple IMU, barometric pressure, motor current draw, and battery state-of-charge (SOC). | `/// OperationalAllocation: [OA-04]`<br>`/// Realises: [UAF-ACT-04]` |
| **OA-05** | `DynamicAirspaceContainment` | Geofence Engine / UTM Client Subsystem | Evaluates 4D spatial boundary proximity against hard geofence limits and U-space dynamic keep-out zones at 50 Hz; enforces automated 180° banking turnaround upon warning. | `/// OperationalAllocation: [OA-05]`<br>`/// Realises: [UAF-ACT-05]` |
| **OA-06** | `AutonomousContingencyManagement` | Deterministic Failsafe State Machine | Detects emergency triggers (`EMG-01`..`EMG-07`), arbitrates priorities, and executes deterministic containment maneuvers (RTH, secondary divert, or parachute deployment) in <= 200 ms. | `/// OperationalAllocation: [OA-06]`<br>`/// Realises: [UAF-ACT-06]` |
| **OA-07** | `SecureDatalinkRelay` | PACE Communications Transceiver Node | Manages multi-tier encryption (AES-256-GCM), monitors link quality (SNR, BER), executes sub-50 ms failover across COFDM, LTE/5G, FHSS, and Iridium satellite links. | `/// OperationalAllocation: [OA-07]`<br>`/// Realises: [UAF-ACT-07]` |
| **OA-08** | `PrecisionRecoveryAndShutdown` | Guidance Subsystem / Ground Station Beacon | Executes automated precision approach using optical fiducial and RTK GNSS landing guidance, flare trajectory, motor cutoff, propeller lock, and post-flight crypto-purge. | `/// OperationalAllocation: [OA-08]`<br>`/// Realises: [UAF-ACT-08]` |

### 6.2 Performer Node Allocation Matrix
The following matrix demonstrates 100% allocation coverage across system nodes:

| System Performer Node | Node Type | Primary Resource | Allocated Activities |
| :--- | :--- | :--- | :--- |
| Air Vehicle Avionics Node | Cyber-Physical Air | Dual Cortex-R5 FCC | OA-01, OA-02, OA-04 |
| Optical/Thermal Payload Node | Sensor Edge Node | Edge Vision AI SoC | OA-03 |
| Safety Watchdog Node | Critical Hardware | Independent FPGA | OA-04, OA-05, OA-06 |
| PACE Transceiver Node | Datalink Router | SDR RF Modem + SIM | OA-07 |
| Ground Control Station Node | Ground Performer | Rugged Workstation | OA-01, OA-05, OA-07, OA-08 |

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
