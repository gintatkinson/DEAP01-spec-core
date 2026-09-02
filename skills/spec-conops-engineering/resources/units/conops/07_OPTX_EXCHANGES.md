| Attribute | Value |
| :--- | :--- |
| **Title** | Operational Information Exchange (Op-Tx) Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Operational Information Exchange (Op-Tx) Matrix

### 7.1 Inter-Node Communication Architecture
The Operational Information Exchange (Op-Tx) architecture defines all mission-critical, control, telemetry, payload, and safety data exchanges between system nodes. The architecture is structured to support deterministic bus protocols (CAN FD, Ethernet IEEE 802.3, RS-422) and wireless RF datalink protocols (COFDM, LTE/5G VPN, FHSS, Iridium SBD) in compliance with NATO STANAG 4586 Annex B (Data Link Interface).

### 7.2 Operational Information Exchange (Op-Tx) Matrix Table
In accordance with OMG UAF v1.2 / v2.0 Operational Information Views (Op-Tx) and the JSON Schema data contract, the information exchanges are specified as follows:

| Exchange ID | Source Node | Destination Node | Information Element / Payload Description | Message Frequency | Nominal Throughput | Peak Throughput | Max Latency Bound | Criticality Level |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **OpTx-01** | PrimarySensorSubsystem | ControllerLogicSubsystem | Raw IMU Delta Angles, Delta Velocities, Barometric Static Pressure, Magnetometer Heading | 400 Hz | 256.0 kbps | 512.0 kbps | 2.5 ms | High (DO-178C DAL-A) |
| **OpTx-02** | ControllerLogicSubsystem | PropulsionActuators | ESC Motor RPM Demand, Servo Deflection PWM Commands, Current Limiter Flags | 200 Hz | 128.0 kbps | 256.0 kbps | 2.5 ms | High (DO-178C DAL-A) |
| **OpTx-03** | GNSSReceiverNode | NavigationFilterSubsystem | Multi-Constellation (GPS/Galileo) PVT Solution, RTK Carrier Phase Residuals, RAIM Status | 20 Hz | 64.0 kbps | 128.0 kbps | 10.0 ms | High (DO-178C DAL-A) |
| **OpTx-04** | OpticalPayloadNode | EdgeVisionProcessor | Uncompressed Raw 4K Video Stream, Gyro Timestamp Frames, Thermal Array Radiometric Matrix | 60 Hz | 1.5 Gbps | 2.0 Gbps | 16.6 ms | Low (DO-178C DAL-D) |
| **OpTx-05** | EdgeVisionProcessor | TelemetryTransceiver | H.265 Encoded Video Stream, Bounding Box Detections, KLV Metadata (STANAG 4609) | 30 Hz | 6.0 Mbps | 12.0 Mbps | 40.0 ms | Medium (DO-178C DAL-C) |
| **OpTx-06** | TelemetryTransceiver | GroundControlStation | Downlink Consolidated Flight Telemetry (Attitude, Altitude, SOC, Link Quality, Geofence Margin) | 50 Hz | 256.0 kbps | 512.0 kbps | 50.0 ms | Medium (DO-178C DAL-B) |
| **OpTx-07** | GroundControlStation | ControllerLogicSubsystem | Uplink C2 Flight Commands, Dynamic Waypoint Updates, ROE Arming Keys, Manual Override Vectors | 20 Hz | 64.0 kbps | 128.0 kbps | 50.0 ms | High (DO-178C DAL-A) |
| **OpTx-08** | SafetyWatchdogNode | TerminationPyrotechnic | Hardware Flight Abort Trigger, Parachute Pyrotechnic Fire Signal, Battery Power Cutoff Strobe | Event-Driven (500 Hz BIT) | 1.0 kbps | 10.0 kbps | 5.0 ms | Critical (DO-254 DAL-A) |

### 7.3 Network Quality of Service (QoS) & Protocol Stack Allocation
To guarantee latency bounds and prevent message collisions across operational buses:
1. **Critical Avionics Bus (CAN FD / ARINC 825):** Transports `OpTx-01`, `OpTx-02`, and `OpTx-03` with deterministic priority arbitration; worst-case bus utilization is restricted to < 42% under full load.
2. **Payload & Vision Bus (Gigabit Ethernet IEEE 802.3ab):** Dedicated point-to-point Ethernet carrying `OpTx-04` and `OpTx-05` isolated from the flight control network via hardware VLAN separation.
3. **Wireless C2 Datalink (STANAG 4586 DLI / COFDM):** Encrypted with AES-256-GCM; incorporates Forward Error Correction (FEC Rate 1/2) and automatic frequency hopping across 16 RF channels.
4. **Safety Watchdog Bus (Optoisolated Discrete Line):** Hardware discrete signal path for `OpTx-08` with dual-switch interlocks to eliminate false-trigger risks while providing guaranteed sub-5 ms actuation.
