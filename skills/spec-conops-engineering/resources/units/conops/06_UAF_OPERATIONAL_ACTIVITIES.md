| Attribute | Value |
| :--- | :--- |
| **Title** | OMG UAF Operational Activity Taxonomy |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 6. OMG UAF Operational Activity Taxonomy

### 6.1 Operational Activity Decomposition
In accordance with OMG Unified Architecture Framework (UAF) v2.0 Operational Processes (Op-Pr) and Operational Taxonomy (Op-Tx), the system operational activities are formally decomposed and allocated 100% to system performer nodes, logical subsystems, and physical resources.

Every operational activity is bound to a machine-verifiable Gate 24 allocation tag (`/// OperationalAllocation: [OA-XX]`) and realization tag (`/// Realises: [UAF-ACT-XX]`).

| Activity ID | Activity Name | Allocated Performer Node & Resource | Operational Description | Traceability & Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- |
| **OA-01** | `PreOperationHealthVerification` | Core Controller / Safety Watchdog | Executes power-on Built-In-Tests (PBIT), cross-checks redundant sensor biases, validates actuator end-stops, verifies cryptographic root of trust, and logs calibration status within t_PBIT <= tau_PBIT_max. | `/// OperationalAllocation: [OA-01]`<br>`/// Realises: [UAF-ACT-01]` |
| **OA-02** | `TransitAndCorridorExecution` | Core Controller / Guidance Subsystem | Computes and tracks multi-dimensional trajectory corridors, executes closed-loop state control at loop rate f_control, and minimizes trajectory tracking error (e_track <= epsilon_track_max). | `/// OperationalAllocation: [OA-02]`<br>`/// Realises: [UAF-ACT-02]` |
| **OA-03** | `PrimaryOperationalProcessing` | Sensor Suite / Edge Processing Node | Captures multi-modal sensor data, executes edge neural feature extraction and state classification, and formats geo-referenced state telemetry. | `/// OperationalAllocation: [OA-03]`<br>`/// Realises: [UAF-ACT-03]` |
| **OA-04** | `SupervisoryTelemetryExchange` | PACE Communications Transceiver Node | Manages bidirectional telemetry exchange, encrypts control commands, and streams consolidated system state data to the supervisory operator station. | `/// OperationalAllocation: [OA-04]`<br>`/// Realises: [UAF-ACT-04]` |
| **OA-05** | `EnvironmentalLimitMonitoring` | Sensor Suite / Safety Watchdog | Continuously evaluates ambient thermal, mechanical, dynamic, and operational boundary parameters against operating limits at monitoring rate f_monitor. | `/// OperationalAllocation: [OA-05]`<br>`/// Realises: [UAF-ACT-05]` |
| **OA-06** | `EnergyResourceManagement` | Power & Resource Management Subsystem | Monitors stored energy/resource state-of-charge (SoC), computes dynamic closed-loop Bingo return thresholds, and regulates power bus distribution. | `/// OperationalAllocation: [OA-06]`<br>`/// Realises: [UAF-ACT-06]` |
| **OA-07** | `ContingencyContainmentExecution` | Deterministic Failsafe State Machine | Detects canonical emergency triggers (`EMG-01`..`EMG-07`), arbitrates priorities, and executes deterministic containment maneuvers within t_resp <= tau_containment. | `/// OperationalAllocation: [OA-07]`<br>`/// Realises: [UAF-ACT-07]` |
| **OA-08** | `PostOperationSecureShutdown` | Core Controller / Security Partition | Executes controlled deceleration to safe state, actuator power isolation, cryptographic data zeroization, and post-operation diagnostic log archival. | `/// OperationalAllocation: [OA-08]`<br>`/// Realises: [UAF-ACT-08]` |

### 6.2 Performer Node Allocation Matrix
The following matrix demonstrates 100% allocation coverage across system nodes:

| System Performer Node | Node Type | Primary Resource | Allocated Activities |
| :--- | :--- | :--- | :--- |
| Core Controller Node | Cyber-Physical Controller | Deterministic Real-Time Core | OA-01, OA-02, OA-08 |
| Sensor Suite Node | Sensor Edge Node | Multi-Modal Sensor Array & Edge Compute | OA-03, OA-05 |
| Safety Watchdog Node | Safety Critical Hardware | Independent Hardware Watchdog | OA-01, OA-05, OA-07 |
| PACE Transceiver Node | Datalink Router | Multi-Channel Communications Modem | OA-04 |
| Resource Management Node | Power / Energy Hub | Smart Battery & BMS Controller | OA-06 |
| Operator Station Node | Operator Performer | Supervisory Operator Console | OA-01, OA-04, OA-08 |

### 6.3 Operational Traceability Invariant
Per DEAP Governance Rule `rules/conops-mission-intent-integrity.md`, all downstream SysML v2 architectural blocks, state machine behavioral charts, and software requirement units must carry direct traceability back to these eight canonical operational activities using the formalized allocation syntax:
- `/// OperationalAllocation: [OA-01]`
- `/// OperationalAllocation: [OA-02]`
- `/// OperationalAllocation: [OA-03]`
- `/// OperationalAllocation: [OA-04]`
- `/// OperationalAllocation: [OA-05]`
- `/// OperationalAllocation: [OA-06]`
- `/// OperationalAllocation: [OA-07]`
- `/// OperationalAllocation: [OA-08]`
