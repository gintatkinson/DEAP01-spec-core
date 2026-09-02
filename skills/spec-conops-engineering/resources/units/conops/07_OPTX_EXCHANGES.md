| Attribute | Value |
| :--- | :--- |
| **Title** | Operational Information Exchange (Op-Tx) Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Operational Information Exchange (Op-Tx) Matrix

### 7.1 Inter-Node Communication Architecture
The Operational Information Exchange (Op-Tx) architecture defines all mission-critical, control, telemetry, payload, and safety data exchanges between system performer nodes. The architecture is structured to support deterministic internal bus protocols and resilient external communication channels in compliance with OMG UAF v2.0 Operational Information Views (Op-Tx).

### 7.2 Operational Information Exchange (Op-Tx) Matrix Table
In accordance with OMG UAF v2.0 Operational Information Views (Op-Tx) and system data contracts, the information exchanges are specified with parametric message rates ($f_{\mathrm{rate}}$), throughputs ($\text{Throughput}$), latency bounds ($\tau_{\mathrm{latency\_max}}$), and criticality levels ($\text{CriticalityLevel}$):

| Exchange ID | Source Node | Destination Node | Information Element / Description | Message Frequency (f_rate) | Nominal Throughput (Throughput) | Max Latency Bound (tau_latency_max) | Criticality Level (CriticalityLevel) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **OpTx-01** | PrimarySensorSuite | CoreController | Raw Sensor Measurements, State Delta Vectors, Environmental Sensor Readings | f_sensor_rate | Throughput_sensor | tau_sensor_latency_max | High (Criticality_High) |
| **OpTx-02** | CoreController | ActuatorSubsystem | Actuator Dynamic Demands, Control Setpoints, Power Limiter Flags | f_actuator_rate | Throughput_actuator | tau_actuator_latency_max | High (Criticality_High) |
| **OpTx-03** | ExternalDataService | CoreController | External Reference Coordinates, Environmental Updates, Deconfliction Status | f_ref_rate | Throughput_ref | tau_ref_latency_max | Medium (Criticality_Medium) |
| **OpTx-04** | SensorSuite | EdgeProcessingNode | High-Bandwidth Sensor Frames, Timestamp Metadata, Matrix Data | f_stream_rate | Throughput_stream | tau_stream_latency_max | Low (Criticality_Low) |
| **OpTx-05** | EdgeProcessingNode | PrimaryCommunications | Extracted Feature Vectors, State Classifications, Telemetry Tags | f_feature_rate | Throughput_feature | tau_feature_latency_max | Medium (Criticality_Medium) |
| **OpTx-06** | PrimaryCommunications | OperatorStation | Downlink Consolidated System Telemetry (State, SoC, Link Quality, Boundary Margins) | f_downlink_rate | Throughput_downlink | tau_downlink_latency_max | Medium (Criticality_Medium) |
| **OpTx-07** | OperatorStation | CoreController | Uplink Supervisory Commands, Dynamic Waypoint Updates, Authorization Tokens | f_uplink_rate | Throughput_uplink | tau_uplink_latency_max | High (Criticality_High) |
| **OpTx-08** | SafetyWatchdog | ActuatorSubsystem | Hardware Safety Abort Trigger, Emergency Safe State Isolation Strobe | Event-Driven (f_watchdog_rate) | Throughput_watchdog | tau_watchdog_latency_max | Critical (Criticality_Critical) |

### 7.3 Network Quality of Service (QoS) & Protocol Stack Allocation
To guarantee latency bounds and prevent message collisions across operational buses:
1. **Critical Control Bus:** Transports `OpTx-01`, `OpTx-02`, and `OpTx-03` with deterministic priority arbitration; worst-case bus utilization is restricted to $\text{Util}_{\mathrm{bus}} \le \text{Util}_{\mathrm{bus\_max}}$ under peak load.
2. **Payload & Processing Bus:** Dedicated high-bandwidth data channel carrying `OpTx-04` and `OpTx-05` logically and electrically isolated from the safety-critical control network.
3. **Supervisory Communication Channel:** Authenticated and encrypted datalink incorporating Forward Error Correction (FEC) and dynamic channel switching across PACE communication tiers.
4. **Safety Watchdog Discrete Line:** Isolated hardware discrete signal path for `OpTx-08` with dual-switch interlocks to eliminate false-trigger risks while providing guaranteed sub-millisecond actuation ($t_{\mathrm{actuate}} \le \tau_{\mathrm{watchdog\_latency\_max}}$).
