| Attribute | Value |
| :--- | :--- |
| **Title** | 7-Row Emergency Decision & Contingency Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 12. 7-Row Emergency Decision & Contingency Matrix

In accordance with MIL-STD-882E §4.3, JARUS SORA v2.5 Annex B, and RTCA DO-178C DAL-A safety requirements, the system incorporates a 7-Row Emergency Decision & Contingency Matrix. The deterministic failsafe state machine provides autonomous containment timing guarantees ($t_{\mathrm{resp}} \le \tau_{\mathrm{containment}}$) across all canonical contingency triggers.

| Trigger ID | Contingency Trigger Event | Anomaly Detection Mechanism | Automated Containment Action | Primary Failsafe State | Max Response Time | HITL Authority Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| EMG-01 | Lost C2 Link | Heartbeat loss duration Delta_t_loss > tau_heartbeat_timeout across active PACE communications tiers | Switches to autonomous lost-link loiter pattern (tau_loiter); if link unrecovered, initiates climb to clearance altitude h_clearance and Return-to-Home (RTH) trajectory | `Contingency_LostLinkReturn` | t_resp <= tau_containment_C2 | Supervisory / Manual Override via Emergency Link |
| EMG-02 | GNSS Navigation Loss | Figure of Merit FOM > FOM_limit, loss of carrier lock, or Receiver Autonomous Integrity Monitoring (RAIM) alert | Disengages GNSS position updates; switches seamlessly to onboard visual-inertial odometry and dead reckoning; sets speed to v_degraded | `Contingency_DeadReckoning` | t_resp <= tau_containment_nav | Supervisory / Command Loiter or Divert |
| EMG-03 | Propulsion Failure | Inverter telemetry over-current I > I_trip, motor RPM drop Delta_RPM > Delta_RPM_trip, or propulsion anomaly | Re-distributes thrust across remaining propulsion units; levels attitude; executes optimal unpowered glide profile to nearest cleared secondary recovery site | `Contingency_EmergencyGlide` | t_resp <= tau_containment_prop | Informed / Automated Advisory |
| EMG-04 | Critical Sensor Fault | Cross-channel disparity Delta_s > Delta_s_threshold across redundant IMU or barometers lasting Delta_t > tau_fault_persist | Isolates faulty sensor channel via voting logic; transitions flight control to simplex sensor failsafe mode; logs fault signature | `Degraded_SensorFailsafe` | t_resp <= tau_containment_sens | Supervisory / Precautionary Recovery Order |
| EMG-05 | Geofence Breach Alert | Navigation filter indicates vehicle proximity d_boundary <= d_containment_margin to hard geofence containment boundary | Commands immediate autonomous maximum-rate coordinated turnaround maneuver (180 deg turn); applies reverse thrust if boundary breached | `Contingency_GeofenceContainment` | t_resp <= tau_containment_geo | Supervisory / Manual Override Authority |
| EMG-06 | Structural Anomaly | Multi-axis accelerometer detects vibration a_vib > a_vib_limit (RMS) or control surface actuator stall | Reduces forward throttle to minimum maneuvering speed v_maneuver_min; initiates immediate controlled precautionary descent | `Contingency_PrecautionaryLand` | t_resp <= tau_containment_struct | Supervisory / Override to Divert |
| EMG-07 | Flight Termination Cmd | Dedicated encrypted manual abort discrete signal received or unrecoverable aerodynamic tumbling (omega_rate > omega_tumble_limit) | Instantly cuts motor power; deploys autonomous ballistic recovery parachute; activates emergency locator beacon | `Emergency_FlightTermination` | t_resp <= tau_containment_term | Initiator (Dual-Consent Authenticated Command) |

### 12.1 Failsafe State Transition Semantics & Timing Guarantees
The emergency decision logic executes within a dedicated, hard real-time safety watchdog running on an isolated hardware partition. The state machine operates under the following deterministic transition invariants:
1. **Priority Arbitration:** In the event of concurrent multiple anomaly triggers, higher-priority states strictly override lower-priority states:

$$
\begin{aligned}
P_{\mathrm{EMG-07}} > P_{\mathrm{EMG-03}} > P_{\mathrm{EMG-05}} > P_{\mathrm{EMG-06}} > P_{\mathrm{EMG-04}} > P_{\mathrm{EMG-02}} > P_{\mathrm{EMG-01}}
\end{aligned}
$$

2. **Deterministic Response Timing ($t_{\mathrm{resp}} \le \tau_{\mathrm{containment}}$):** From initial sensor anomaly threshold crossing to containment command issuance on the actuator bus, the maximum propagation latency is guaranteed to remain strictly bounded by $t_{\mathrm{resp}} \le \tau_{\mathrm{containment}}$ across all seven canonical triggers (with flight termination executing within $t_{\mathrm{resp}} \le \tau_{\mathrm{containment\_term}}$).
3. **Fail-Safe Retention:** Once a critical emergency state (`Emergency_FlightTermination` or `Contingency_EmergencyGlide`) is triggered, the state machine is non-reentrant and locks until a physical post-flight ground reset and authorized Maintenance Technician clearance are performed.
