| Attribute | Value |
| :--- | :--- |
| **Title** | Current Situation, Deficiency Analysis & Operational Motivation |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 2. Current Situation, Deficiency Analysis & Operational Motivation

### 2.1 Current Operational Baseline (Predecessors)
The predecessor operational baseline relies on conventional radio-controlled (RC) unmanned platforms and legacy point-to-point analog/digital video telemetry systems. These systems were primarily designed for line-of-sight (VLOS) remote piloting under benign weather conditions, characterized by:
1. **Single-String Non-Redundant Avionics:** Predecessor platforms utilize commercial off-the-shelf (COTS) flight controllers lacking redundant inertial sensors, dual power buses, and hardware watchdog circuits. A single sensor anomaly or processor stall leads directly to unrecoverable loss of control.
2. **Manual Point-to-Point Control Links:** Command and control relies on single-frequency analog or low-resilience digital radio links (e.g., 2.4 GHz or 5.8 GHz ISM) vulnerable to multipath fading, intentional electronic warfare (EW) jamming, and line-of-sight terrain obstruction.
3. **Disjointed Sensor Telemetry Displays:** Payload imagery and flight navigational telemetry are processed on isolated, unintegrated display monitors, forcing human operators to perform manual mental correlation of video feeds with spatial maps.
4. **Ad-Hoc Contingency Management:** Contingency responses (such as simple motor kill or uncontrolled descent) lack deterministic timing guarantees, formal SORA containment buffers, and dynamic secondary divert routing.

### 2.2 Operational Capability Gaps
Analysis of historical mission logs, field incident reports, and operator debriefs reveals four critical capability gaps that prevent legacy systems from executing high-tempo autonomous perimeter surveillance:

| Gap ID | Capability Gap Name | Operational Manifestation | Mission Impact & Risk |
| :--- | :--- | :--- | :--- |
| **GAP-01** | Safe BVLOS Autonomous Operation | Inability to maintain statutory airspace separation and geofence containment without constant visual observer line-of-sight. | Restricted operational radius (< 1.5 km), preventing wide-area perimeter monitoring. |
| **GAP-02** | Contested RF / EW Resilience | Loss of vehicle control and video telemetry in the presence of intentional GNSS jamming, RF interference, or urban multipath. | High rate of lost-link aborts, mission abandonment, and potential flyaway hazards. |
| **GAP-03** | Sub-Second Deterministic Failsafe | Slow anomaly detection (> 5.0 s) and non-deterministic recovery maneuvers resulting in ground impact boundary breaches. | Inability to achieve JARUS SORA Specific Category authorization for populated adjacent areas. |
| **GAP-04** | Modular Rapid Field Turnaround | Prolonged battery charging and complex multi-tool payload swaps requiring > 45 minutes of ground turnaround between sorties. | Sortie rate limited to < 4 flights per operational shift, leaving perimeter security coverage gaps. |

### 2.3 Legacy System Deficiencies & Technical Limitations
In accordance with ISO/IEC/IEEE 29148:2018 §5.2.4, system deficiencies are categorized across technical, operational, and human domains:

| Technical Domain | Operational Domain | Human Domain |
| :--- | :--- | :--- |
| Single-string IMU | No dynamic geofencing | High NASA-TLX load |
| Lack of RTOS bounds | No automated U-space link | Disjointed displays |
| Unshielded EMI/RF | Manual flight log extract | Fatiguing piloting |
| Non-IP67 enclosures | 45-minute turnaround | Complex calibration |

1. **Technical Deficiencies:**
   - **Avionics & Compute:** Lack of hard real-time operating system (RTOS) guarantees; flight control loops executed on non-deterministic microcontrollers subject to task starvation and priority inversion.
   - **Environmental Hardening:** Non-sealed enclosures (rated IP43 or lower) vulnerable to wind-driven rain, sand ingress, and salt-fog corrosion, preventing operation in harsh climatic conditions (-40°C to +55°C).
   - **Power Distribution:** Single battery rail architecture without isolation diodes; short-circuit in payload subsystem instantly collapses primary flight avionics bus.

2. **Operational Deficiencies:**
   - **Airspace Integration:** Complete lack of automated communication with U-space / UTM service providers, necessitating manual phone calls to local air traffic control for airspace coordination.
   - **Contingency Divert:** Pre-programmed Return-to-Home (RTH) trajectories follow straight-line paths ignoring active terrain obstacles, dynamic weather cells, and pop-up keep-out zones.

3. **Human Deficiencies:**
   - **Cognitive Overload:** Operators are required to simultaneously pilot the aircraft, steer the optical sensor, monitor battery telemetry, and log airspace coordinates manually, leading to NASA-TLX mental workload scores exceeding 75/100.
   - **Operational Error:** Fatigued operators during extended nighttime shifts exhibit increased reaction times (> 3.0 s) to contingency warnings, leading to preventable incidents.

### 2.4 Mission Drivers & User Operational Problems
The primary mission drivers compelling the development of the Autonomous Cyber-Physical System Archetype include:
- **Statutory Regulatory Mandates:** Civil aviation authorities (FAA, EASA) and defense agencies require formal compliance with JARUS SORA v2.5 Annex B for operations beyond visual line-of-sight. This requires verifiable Ground Risk Buffer calculations and DO-178C DAL-A/B software assurance.
- **Continuous 24/7 Security Coverage:** Critical infrastructure protection requires unbroken surveillance coverage over a 15 km perimeter with autonomous day/night handoffs between airborne assets.
- **Adverse Environmental Readiness:** The system must operate reliably across extreme climatic envelopes (-40°C Arctic frost to +55°C desert heat, IP67 ingress sealing, and 15 m/s continuous wind gusts per MIL-STD-810H).
- **Zero-Trust Cybersecurity Baseline:** Modern operational environments demand AES-256 encrypted C2 command links, hardware security module (HSM) key storage, and automated rejection of spoofed telemetry or malicious command injection.
