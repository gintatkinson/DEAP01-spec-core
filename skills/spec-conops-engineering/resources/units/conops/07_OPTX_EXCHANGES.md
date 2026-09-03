| Attribute | Value |
| :--- | :--- |
| **Title** | Operational Information Exchange (Op-Tx) Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Operational Information Exchange (Op-Tx) Matrix

### 7.1 Inter-Node Communication Architecture
The Operational Information Exchange (Op-Tx) architecture defines all mission-critical, flight control, telemetry, payload, external coordination, and safety data exchanges between system performer nodes. The architecture is structured to support deterministic internal bus protocols, high-throughput payload fabrics, wireless RF datalink channels, direct broadcast identification, and hardwired failsafe discrete lines in compliance with OMG Unified Architecture Framework (UAF) v2.0 Operational Information Views (Op-Tx) and ISO/IEC/IEEE 29148:2018.

### 7.2 Operational Information Exchange (Op-Tx) Matrix Table
In accordance with OMG UAF v2.0 Operational Information Views (Op-Tx), RTCA DO-178C / DO-254 design assurance levels (DAL), and system data contracts, the operational information exchanges are specified across 16 canonical interaction channels:

| Exchange ID | Information Exchange Name | Source Performer | Destination Performer | Exchange Item & Semantic Content | Trigger Mechanism | Nominal Frequency (Hz) | Peak Throughput | Max Latency Bound (tau_max ms) | Integrity Assurance Level | Security Protection |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **OpTx-01** | PrimarySensorTelemetry | PrimarySensorSuite | CoreController | Raw Inertial & Kinematic Measurements, State Delta Vectors, Barometric Static Pressure, Magnetometer Heading | Periodic (Deterministic Timer) | f_sensor_rate | Throughput_sensor | tau_sensor_latency_max | DAL-A | Dual-Bus Redundancy & CRC32 Frame Integrity Validation |
| **OpTx-02** | ActuatorControlDemand | CoreController | ActuatorSubsystem | Dynamic Torque Demands, Control Surface Deflections, Speed Controller Setpoints, Power Limiter Flags | Periodic (Control Loop Execution) | f_actuator_rate | Throughput_actuator | tau_actuator_latency_max | DAL-A | Hardware Interlock & Cyclic Frame Counter Verification |
| **OpTx-03** | ActuatorStateFeedback | ActuatorSubsystem | CoreController | Measured Actuator Positions, Motor RPM Telemetry, Phase Current Draw, Thermal Diagnostic Flags | Periodic (Feedback Loop Sampling) | f_actuator_fb_rate | Throughput_actuator_fb | tau_actuator_fb_latency_max | DAL-B | CRC32 Frame Validation & Disparity Threshold Checking |
| **OpTx-04** | PowerResourceTelemetry | PowerManagementSubsystem | CoreController | Total Battery Pack Voltage, Cell Temperature Matrix, State-of-Charge (SoC), Current Draw, Dynamic Bingo Thresholds | Periodic (BMS Monitor Interval) | f_bms_rate | Throughput_bms | tau_bms_latency_max | DAL-B | Hardware Isolated Bus & Range Bound Sanitization |
| **OpTx-05** | ExternalNavReferenceData | ExternalPositioningService | CoreController | Multi-Constellation Satellite PVT Solutions, RTK Differential Phase Residuals, Ephemeris Data, UTC Time Reference | Periodic (Epoch Arrival) | f_nav_ref_rate | Throughput_nav_ref | tau_nav_ref_latency_max | DAL-B | RAIM Integrity Monitoring & Ephemeris Sanity Verification |
| **OpTx-06** | RawPayloadSensorStream | SensorSuite | PayloadSubsystem | High-Bandwidth Multi-Modal Video Frames, Timestamp Metadata, Radiometric Matrices, Raw Spatial Point Clouds | Continuous (Frame Clock Synchronized) | f_stream_rate | Throughput_stream | tau_stream_latency_max | DAL-D | Dedicated DMA Channel & Isolated Subnet VLAN |
| **OpTx-07** | ProcessedFeatureTelemetry | PayloadSubsystem | CoreController | Extracted State Feature Vectors, Target Bounding Boxes, Optical Odometry Vectors, Environmental Obstacle Disparities | Periodic (Inference Pipeline Execution) | f_feature_rate | Throughput_feature | tau_feature_latency_max | DAL-C | Memory Partition Isolation & Ingress Range Sanitization |
| **OpTx-08** | ConsolidatedDownlinkTelemetry | PrimaryCommunications | OperatorStation | Consolidated System State Telemetry, 3D Kinematics, Energy SoC, Communications Link SNR, Geofence Margin Status | Periodic (PACE Telemetry Scheduler) | f_downlink_rate | Throughput_downlink | tau_downlink_latency_max | DAL-B | Authenticated Encryption (AES-256-GCM / TLS 1.3) & HMAC-SHA256 |
| **OpTx-09** | SupervisoryUplinkCommand | OperatorStation | CoreController | Uplink Flight Directives, Dynamic 4D Waypoint Corridors, ROE Arming Keys, Manual Override Mode Vectors | Aperiodic (Operator Command Action) | f_uplink_rate | Throughput_uplink | tau_uplink_latency_max | DAL-A | Dual-Signature Token Authentication & Nonce Replay Protection |
| **OpTx-10** | WatchdogHeartbeatStrobe | CoreController | SafetyWatchdog | Controller Task Alive Token, Execution Deadline Checksum, Task Schedule Monotonic Counter | Periodic (Task Deadline Checkpoint) | f_watchdog_hb_rate | Throughput_watchdog_hb | tau_watchdog_hb_latency_max | DAL-A | Dedicated Hardware Strobe Line & Timing Window Interlock |
| **OpTx-11** | EmergencyFailsafeTrigger | SafetyWatchdog | ActuatorSubsystem | Hardware Safety Abort Strobe, Power Stage Isolation Command, Safe State Clamping Signal | Event-Driven (EMG-01..EMG-07 Fault Detection) | Event-Driven (f_watchdog_rate) | Throughput_watchdog | tau_watchdog_latency_max | DAL-A | Hardware Interlock & Dual-Switch Isolation Lines |
| **OpTx-12** | FailsafeSquibActuationLine | SafetyWatchdog | EmergencyContainmentSubsystem | Dual-Channel High-Current Fire Pulse, {{CONTAINMENT_SQUIB_ACTION:Parachute / Pyrotechnic Cutter Ignition Command}} | Event-Driven (EMG-07 / Critical Boundary Breach) | Event-Driven (Single Pulse Strobe) | Discrete Pulse | tau_squib_latency_max | DAL-A | Isolated High-Side/Low-Side Solid-State Switches & Physical Key Interlock |
| **OpTx-13** | {{OPTX13_NAME:BroadcastRemoteIDTelemetry}} | {{OPTX13_SOURCE:BroadcastRemoteID}} | ExternalDataService | Statutory Broadcast ID, Serial Number / Session ID, Geodetic Position Coordinates, {{ALTITUDE_TELEMETRY:Altitude}}, Ground Speed, Emergency Status | Periodic (Statutory Broadcast Scheduler) | f_remote_id_rate | Throughput_remote_id | tau_remote_id_latency_max | DAL-C | {{OPTX13_PROTOCOL_DESC:Digitally Signed Public Broadcast (Bluetooth 5.x / Wi-Fi Beacon per ASTM F3411-22a)}} |
| **OpTx-14** | AirspaceDeconflictionData | ExternalDataService | OperatorStation | Dynamic Geo-Zone Activation/Deactivation, Strategic 4D Corridor Clearances, Adjacent Traffic Conformance Status | Periodic & Event-Driven (UTM Service Updates) | f_utm_rate | Throughput_utm | tau_utm_latency_max | DAL-C | Mutual TLS (mTLS 1.3) & REST/JSON Schema Signature Verification |
| **OpTx-15** | CompressedPayloadVideoStream | PayloadSubsystem | OperatorStation | H.264/H.265 Encoded Video Stream, KLV Metadata (MISB ST 0601 / STANAG 4609), Real-Time Feature Overlays | Continuous (RTP/RTSP Video Frame Pipeline) | f_video_rate | Throughput_video | tau_video_latency_max | DAL-D | SRTP / AES-CTR Encryption over Wireless PACE Primary/Alternate Tiers |
| **OpTx-16** | DiagnosticBlackboxLogStream | CoreController | OperatorStation | High-Rate Flight Recorder Time-Series, Sensor Disparity Registers, Exception Stack Traces, BIT Failure Logs | Periodic Buffering & Post-Operation Bulk Transfer | f_log_rate | Throughput_log | tau_log_latency_max | DAL-C | Cryptographic Hashing (SHA-256), Write-Once Flash Partition, Append-Only Non-Volatile Storage |

### 7.3 Avionic Network Quality of Service (QoS) Stack Allocation
To guarantee deterministic latency bounds, eliminate message collisions, and enforce strict physical and logical partitioning across all operational interactions, the 16 Op-Tx exchanges are allocated across five distinct avionic physical and logical network tiers:

#### 7.3.1 Deterministic Real-Time Bus (CAN FD / TTP / ARINC 825)
- **Allocated Exchanges:** `OpTx-01`, `OpTx-02`, `OpTx-03`, `OpTx-04`, `OpTx-05`, `OpTx-07`, and `OpTx-10`.
- **Protocol Profile & Physical Medium:** Controller Area Network Flexible Data-Rate (CAN FD per ISO 11898-1:2015, up to 5.0 Mbps, 64-byte payload) / Time-Triggered Protocol (TTP per SAE AS6003) / ARINC 825 avionic CAN standard.
- **Deterministic Arbitration & Schedulability Guarantees:** Fixed-priority non-preemptive arbitration based on 29-bit extended CAN identifier message IDs. Maximum worst-case bus utilization is bounded by:

$$
\begin{aligned}
\text{Util}_{\text{bus}} &= \sum_{i=1}^{N_{\text{bus}}} \frac{C_i}{T_i} \le \text{Util}_{\text{bus\_max}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
- $\text{Util}_{\text{bus}}$: Total deterministic real-time bus utilization under worst-case burst conditions.
- $N_{\text{bus}}$: Total number of active periodic message streams allocated to the deterministic bus ($N_{\text{bus}} = 7$).
- $C_i$: Worst-case transmission time for message stream $i$ including bit-stuffing overhead.
- $T_i$: Minimum period of message stream $i$ ($T_i = 1 / f_{\text{rate}, i}$).
- $\text{Util}_{\text{bus\_max}}$: Maximum allowable bus utilization ceiling ($\text{Util}_{\text{bus\_max}} \le 0.60$), ensuring a minimum $40\%$ bandwidth margin for asynchronous error handling and network management frames.
- **Redundancy & Fault Isolation:** Dual-channel physical transceivers (Bus Alpha and Bus Bravo) operating in hot-standby with automatic babbling-node isolation, hardware loopback verification, and failover latency $t_{\text{failover}} \le \tau_{\text{bus\_failover\_max}}$.

#### 7.3.2 High-Speed Payload Bus (PCIe / Gigabit Ethernet / TSN IEEE 802.1Qbv)
- **Allocated Exchanges:** `OpTx-06`, `OpTx-15`, and `OpTx-16`.
- **Protocol Profile & Physical Medium:** IEEE 802.3ab 1000BASE-T Gigabit Ethernet with Time-Sensitive Networking (TSN IEEE 802.1Qbv Scheduled Traffic and IEEE 802.1Qav Credit-Based Shaper) / Peripheral Component Interconnect Express (PCIe Gen3/4 with DMA).
- **QoS Partitioning & Bandwidth Guarantees:** Physical and logical segregation (dedicated Ethernet PHY and IEEE 802.1Q VLANs) completely isolates high-bandwidth sensor payload data from flight-critical control traffic. High-capacity zero-copy ring buffers and Direct Memory Access (DMA) ensure sustained payload throughput ($\text{Throughput}_{\text{payload\_bus}} \ge 1.0\text{ Gbps}$) without inducing processor starvation or memory bus contention on the Core Controller.

#### 7.3.3 Telemetry Wireless PACE Links (Primary, Alternate, Contingency, Emergency)
- **Allocated Exchanges:** `OpTx-08`, `OpTx-09`, `OpTx-14`, and `OpTx-15`.
- **Multi-Tier PACE Communication Stack:**
  1. **Primary Link (COFDM Point-to-Point / 5.8 GHz ISM):** High-throughput channel carrying consolidated flight telemetry (`OpTx-08`) and real-time compressed video (`OpTx-15`) with nominal bandwidth $\ge 10.0\text{ Mbps}$ and transport latency $\le \tau_{\text{Primary\_max}}$.
  2. **Alternate Link (Cellular LTE/5G Encrypted VPN / Broadband Satcom):** Secure routed IP tunnel carrying telemetry (`OpTx-08`), UTM coordination updates (`OpTx-14`), and supervisory commands (`OpTx-09`) with bandwidth $\ge 2.0\text{ Mbps}$ and transport latency $\le \tau_{\text{Alternate\_max}}$.
  3. **Contingency Link (900 MHz FHSS Narrowband Radio):** Robust frequency-hopping datalink dedicated exclusively to essential C2 flight directives (`OpTx-09`) and heartbeat signals (`OpTx-08`) with bandwidth $\ge 115.2\text{ kbps}$ and transport latency $\le \tau_{\text{Contingency\_max}}$.
  4. **Emergency Link (Satellite Iridium SBD / Low-Frequency Beacon):** Ultra-reliable channel providing global beacon broadcast and dual-consent flight termination confirmations with bandwidth $\ge 2.4\text{ kbps}$ and transport latency $\le \tau_{\text{Emergency\_max}}$.
- **QoS Failover & Security Protection:** Automated link quality monitor evaluating signal-to-noise ratio ($\text{SNR}$), packet loss rate ($\text{PLR}$), and heartbeat timeouts ($t_{\mathrm{loss}} > \tau_{\mathrm{timeout}}$). Failover transitions enforce hysteresis time $\Delta t_{\mathrm{hysteresis}}$ to prevent link flapping. All command channels enforce AES-256-GCM encryption, HMAC-SHA256 authentication, and anti-replay nonce tracking per NIST SP 800-82r3.

#### 7.3.4 {{REMOTE_ID_HEADER:ASTM F3411 Direct Broadcast Remote ID}}
- **Allocated Exchanges:** `OpTx-13`.
- **Protocol Profile & Broadcast Mechanism:** {{REMOTE_ID_STANDARD_BODY:Direct connectionless RF broadcast via Bluetooth 5.x Long Range (LE Coded PHY) and Wi-Fi Beacon Frames (IEEE 802.11 Direct Broadcast) in accordance with ASTM F3411-22a and ASD-STAN prEN 4709-002 standards.}}
- **QoS & Timing Determinism:** Autonomous periodic broadcast scheduler issuing signed identification packets at $f_{\text{remote\_id\_rate}} = 1.0\text{ Hz}$ to $2.0\text{ Hz}$ with latency $\tau_{\text{remote\_id}} \le 200\text{ ms}$. Time synchronization referenced to UTC via GNSS epoch. Independent RF front-end prevents resource exhaustion by external datalinks, guaranteeing continuous identification broadcast even during lost-link contingency states.

#### 7.3.5 Hardwired Failsafe Squib Lines & Safety Discretes
- **Allocated Exchanges:** `OpTx-11` and `OpTx-12`.
- **Physical & Electrical Topology:** Optoisolated, point-to-point discrete wiring and high-current solid-state power switches directly routed between the independent Safety Watchdog and emergency containment / actuator isolation hardware.
- **Safety Interlock Architecture:** Dual-switch configuration incorporating independent high-side solid-state switch and low-side ground clamp with pull-down resistors. Actuation requires simultaneous coincidence of hardware interlock permissive signals and Safety Watchdog abort triggers, eliminating false-trigger vulnerability to EMI transients or processor brownouts. Guarantees deterministic sub-millisecond actuation ($t_{\text{actuate}} \le \tau_{\text{squib\_latency\_max}} \le 10\text{ ms}$) triggering {{FAILSAFE_DESCENT_SYSTEM:ballistic recovery deployment}}, propulsion power cutoff, or emergency safe-state clamping.
