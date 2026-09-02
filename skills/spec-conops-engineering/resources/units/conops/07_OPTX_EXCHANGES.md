| Attribute | Value |
| :--- | :--- |
| **Title** | Operational Information Exchange (Op-Tx) Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Operational Information Exchange (Op-Tx) Matrix

### 7.1 Inter-Node Communication Architecture
The Operational Information Exchange (Op-Tx) architecture defines all mission-critical, control, telemetry, payload, and safety data exchanges between system nodes. The architecture is structured to support deterministic internal bus protocols and wireless RF datalink protocols in compliance with NATO STANAG 4586 Annex B (Data Link Interface).

### 7.2 Operational Information Exchange (Op-Tx) Matrix Table
In accordance with OMG UAF v1.2 / v2.0 Operational Information Views (Op-Tx) and the JSON Schema data contract, the information exchanges are specified with parametric message rates, throughputs, latency bounds, and Design Assurance Levels (DAL):

| Exchange ID | Source Node | Destination Node | Information Element / Payload Description | Message Frequency (f_sample) | Nominal Throughput (Throughput_nom) | Max Latency Bound (tau_max) | Criticality Level (DAL_req) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **OpTx-01** | PrimarySensorSubsystem | ControllerLogicSubsystem | Raw IMU Delta Angles, Delta Velocities, Barometric Static Pressure, Magnetometer Heading | f_sensor_sample | Throughput_sensor_nom | tau_sensor_max | High (DO-178C DAL-A) |
| **OpTx-02** | ControllerLogicSubsystem | PropulsionActuators | Motor RPM Demands, Actuator Deflection Commands, Current Limiter Flags | f_actuator_sample | Throughput_actuator_nom | tau_actuator_max | High (DO-178C DAL-A) |
| **OpTx-03** | GNSSReceiverNode | NavigationFilterSubsystem | Multi-Constellation Satellite PVT Solution, Carrier Phase Residuals, RAIM Integrity Status | f_nav_sample | Throughput_nav_nom | tau_nav_max | High (DO-178C DAL-A) |
| **OpTx-04** | OpticalPayloadNode | EdgeVisionProcessor | Raw Video Stream Frames, Timestamp Metadata, Radiometric Thermal Matrix | f_video_sample | Throughput_video_nom | tau_video_max | Low (DO-178C DAL-D) |
| **OpTx-05** | EdgeVisionProcessor | TelemetryTransceiver | Encoded Video Stream, Target Bounding Boxes, Feature Detections, KLV Geospatial Metadata | f_telemetry_sample | Throughput_telemetry_nom | tau_telemetry_max | Medium (DO-178C DAL-C) |
| **OpTx-06** | TelemetryTransceiver | GroundControlStation | Downlink Consolidated Flight Telemetry (Attitude, Altitude, Energy SoC, Link Quality, Geofence Margin) | f_downlink_sample | Throughput_downlink_nom | tau_downlink_max | Medium (DO-178C DAL-B) |
| **OpTx-07** | GroundControlStation | ControllerLogicSubsystem | Uplink C2 Flight Commands, Dynamic Waypoint Updates, ROE Arming Keys, Manual Override Vectors | f_uplink_sample | Throughput_uplink_nom | tau_uplink_max | High (DO-178C DAL-A) |
| **OpTx-08** | SafetyWatchdogNode | TerminationPyrotechnic | Hardware Flight Abort Trigger, Emergency Recovery Deploy Signal, Power Bus Isolation Strobe | Event-Driven (f_watchdog) | Throughput_watchdog_nom | tau_watchdog_max | Critical (DO-254 DAL-A) |

### 7.3 Network Quality of Service (QoS) & Protocol Stack Allocation
To guarantee latency bounds and prevent message collisions across operational buses:
1. **Critical Avionics Bus:** Transports `OpTx-01`, `OpTx-02`, and `OpTx-03` with deterministic priority arbitration; worst-case bus utilization is restricted to $\text{Util}_{\mathrm{bus}} \le \text{Util}_{\mathrm{bus\_max}}$ under peak load.
2. **Payload & Vision Bus:** Dedicated high-bandwidth data channel carrying `OpTx-04` and `OpTx-05` logically and electrically isolated from the safety-critical flight control network.
3. **Wireless C2 Datalink:** Authenticated and encrypted datalink incorporating Forward Error Correction (FEC) and dynamic channel switching across PACE communication tiers.
4. **Safety Watchdog Discrete Line:** Isolated hardware discrete signal path for `OpTx-08` with dual-switch interlocks to eliminate false-trigger risks while providing guaranteed sub-millisecond actuation ($t_{\mathrm{actuate}} \le \tau_{\mathrm{watchdog\_max}}$).
