| Attribute | Value |
| :--- | :--- |
| **Title** | 7-Row Emergency Decision & Contingency Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 12. 7-Row Emergency Decision & Contingency Matrix

In accordance with MIL-STD-882E §4.3, JARUS SORA v2.5 Annex B, and RTCA DO-178C DAL-A safety requirements, the system incorporates a 7-Row Emergency Decision & Contingency Matrix. The deterministic failsafe state machine provides sub-200 ms autonomous containment guarantees across all canonical contingency triggers.

| Trigger ID | Contingency Trigger Event | Anomaly Detection Mechanism | Automated Containment Action | Primary Failsafe State | Max Response Time | HITL Authority Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| EMG-01 | Lost C2 Link | C2 heartbeat timeout > 5.0 s across all active PACE communications tiers (COFDM, Cellular, FHSS) | Switches to autonomous lost-link loiter pattern (30 s); if link unrecovered, initiates direct climb-to-clearance and Return-to-Home (RTH) trajectory | `Contingency_LostLinkReturn` | 0.20 s | Supervisory / Manual Override via Satellite Beacon |
| EMG-02 | GNSS Navigation Loss | Multi-constellation Figure of Merit (FOM) > 3.0, loss of carrier lock, or Receiver Autonomous Integrity Monitoring (RAIM) alert | Disengages GNSS position updates; switches seamlessly to onboard visual-inertial odometry and optical flow dead reckoning; reduces cruising speed to 12.0 m/s | `Contingency_DeadReckoning` | 0.10 s | Supervisory / Command Loiter or Divert |
| EMG-03 | Propulsion Failure | ESC telemetry phase over-current > 45 A, motor RPM drop > 25% under steady command, or single-motor failure | Re-distributes thrust across remaining propulsion units; levels attitude; executes optimal unpowered glide profile to nearest cleared secondary recovery site | `Contingency_EmergencyGlide` | 0.05 s | Informed / Automated Advisory |
| EMG-04 | Critical Sensor Fault | Cross-channel disparity (> 3.0 sigma) across triple-redundant IMU or dual static barometers lasting > 50 ms | Isolates faulty sensor channel via hardware voting logic; transitions flight control to simplex sensor failsafe mode; logs fault signature | `Degraded_SensorFailsafe` | 0.05 s | Supervisory / Precautionary Recovery Order |
| EMG-05 | Geofence Breach Alert | Navigation filter indicates vehicle proximity < 50.0 m to hard geofence containment boundary | Commands immediate autonomous 180° maximum-rate coordinated banking turnaround maneuver; applies maximum reverse thrust if boundary breached | `Contingency_GeofenceContainment` | 0.15 s | Supervisory / Manual Override Authority |
| EMG-06 | Structural Anomaly | Multi-axis accelerometer detects continuous airframe vibration > 2.5 g (RMS) or control surface servo stall | Reduces forward throttle to minimum maneuvering speed; initiates immediate controlled precautionary vertical descent and touchdown on cleared terrain | `Contingency_PrecautionaryLand` | 0.20 s | Supervisory / Override to Divert |
| EMG-07 | Flight Termination Cmd | Dedicated encrypted manual abort discrete signal received or unrecoverable aerodynamic tumbling (roll rate > 180 deg/s) | Instantly cuts all motor power; fires pyrotechnic ballistic recovery parachute; activates 406 MHz emergency locator transmitter (ELT) beacon | `Emergency_FlightTermination` | 0.02 s | Initiator (Two-Person Authenticated Command) |

### 12.1 Failsafe State Transition Semantics & Timing Guarantees
The emergency decision logic executes within a dedicated, hard real-time safety watchdog running on an isolated FPGA / Cortex-R5 partition. The state machine operates under the following deterministic transition invariants:
1. **Priority Arbitration:** In the event of concurrent multiple anomaly triggers, higher-priority states strictly override lower-priority states:
$$
\begin{aligned}
P_{\mathrm{EMG-07}} > P_{\mathrm{EMG-03}} > P_{\mathrm{EMG-05}} > P_{\mathrm{EMG-06}} > P_{\mathrm{EMG-04}} > P_{\mathrm{EMG-02}} > P_{\mathrm{EMG-01}}
\end{aligned}
$$
2. **Deterministic Response Timing ($t_{\text{resp}} \le 200\text{ ms}$):** From initial sensor anomaly threshold crossing to containment command issuance on the actuator bus, the maximum propagation latency is guaranteed to remain strictly $\le 200\text{ ms}$ across all seven canonical triggers (with catastrophic flight termination executing in $\le 20\text{ ms}$).
3. **Fail-Safe Retention:** Once a critical emergency state (`Emergency_FlightTermination` or `Contingency_EmergencyGlide`) is triggered, the state machine is non-reentrant and locks until a physical post-flight ground reset and Maintenance Technician authorization are performed.
