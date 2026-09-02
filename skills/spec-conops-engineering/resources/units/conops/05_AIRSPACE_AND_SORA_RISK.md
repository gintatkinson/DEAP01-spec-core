| Attribute | Value |
| :--- | :--- |
| **Title** | 4D Operational Volume & SORA Risk Assessment |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 5. 4D Operational Volume & SORA v2.5 Risk Assessment

### 5.1 4D Spatial-Temporal Operational Volume Formulation
The operational airspace volume is formally defined as a bounded 4D spatial-temporal continuum consisting of the nominal Flight Geometry, the Contingency Volume, and the Ground Risk Buffer (GRB). In accordance with JARUS SORA v2.5 Annex B, the total 4D volume envelope and the minimum Ground Risk Buffer radius are formulated as:

$$
\begin{aligned}
V_{\mathrm{4D}} &= V_{\mathrm{SpatialGeometry}} \cup V_{\mathrm{ContingencyVolume}} \cup V_{\mathrm{GRB}} \\
R_{\mathrm{GRB}} &= v_{\mathrm{max}} \cdot t_{\mathrm{resp}} + \frac{v_{\mathrm{max}}^2}{2 \cdot a_{\mathrm{decel}}} + h_{\mathrm{ceiling}} \cdot \tan(\theta_{\mathrm{glide}}) + d_{\mathrm{containment}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max Operating Ceiling | h_ceiling | m | h_ceiling <= h_operational_max | Maximum operating ceiling above reference ground surface |
| Maximum Operational Velocity | v_max | m/s | v_max <= v_envelope_max | Maximum forward operational velocity |
| Containment Response Time | t_resp | s | t_resp <= tau_containment_req | Maximum time from anomaly detection to actuator containment execution |
| Vehicle Deceleration Rate | a_decel | m/s^2 | a_decel >= a_decel_min | Minimum operational deceleration capability under dynamic braking |
| Worst-Case Glide Angle | theta_glide | deg | 0 < theta_glide < 90 | Worst-case unpowered aerodynamic descent trajectory angle |
| Containment Buffer Margin | d_containment | m | d_containment >= d_margin_min | Contingency buffer accounting for navigation uncertainty and wind drift |
| Ground Risk Buffer Radius | R_GRB | m | R_GRB >= R_GRB_min | Declared ground risk buffer lateral containment radius |
| System Total Mass | m_system | kg | m_system <= m_MTOW | Total operational takeoff mass of the cyber-physical system |
| Terminal Fall Velocity | v_terminal | m/s | v_terminal = sqrt(2 * m_system * g / (rho * C_D * A_ref)) | Unpowered vertical / ballistic aerodynamic terminal velocity |
| Impact Kinetic Energy | E_k | J | E_k = 0.5 * m_system * v_terminal^2 | Kinetic energy at operational boundary ground impact |
| Ground Risk Classification | GRC | Dimensionless | GRC in {GRC-1, ..., GRC-7} | JARUS SORA v2.5 intrinsic ground risk class rating |

The symbolic kinematic derivation for $R_{\mathrm{GRB}}$ accounts for:
- Reaction distance during anomaly detection and command execution: $d_{\mathrm{reaction}} = v_{\mathrm{max}} \cdot t_{\mathrm{resp}}$
- Dynamic braking deceleration distance: $d_{\mathrm{decel}} = \frac{v_{\mathrm{max}}^2}{2 \cdot a_{\mathrm{decel}}}$
- Unpowered ballistic/glide descent horizontal translation: $d_{\mathrm{glide}} = h_{\mathrm{ceiling}} \cdot \tan(\theta_{\mathrm{glide}})$
- Geometric margin for navigation filter uncertainty and ambient wind drift: $d_{\mathrm{containment}}$
- Declared $R_{\mathrm{GRB}}$ satisfies $R_{\mathrm{GRB}} \ge d_{\mathrm{reaction}} + d_{\mathrm{decel}} + d_{\mathrm{glide}} + d_{\mathrm{containment}}$, providing a mathematically verified containment envelope.

### 5.2 JARUS SORA v2.5 Ground Risk Class (GRC) & Kinetic Impact Limits
1. **Intrinsic Ground Risk Class (Initial GRC):**
   - Maximum characteristic dimension: $L_{\mathrm{char}}$.
   - Typical operational cruising velocity: $v_{\mathrm{cruise}}$.
   - Operational scenario: Over controlled ground perimeter with adjacent low-density population.
   - Intrinsic Ground Risk Class: Mapped to GRC baseline per SORA v2.5 Table 2.

2. **Kinetic Impact Energy Analysis ($E_k$):**
   - Unmitigated kinetic energy formulation at aerodynamic terminal velocity:

$$
\begin{aligned}
E_k &= \frac{1}{2} m_{\mathrm{system}} v_{\mathrm{terminal}}^2
\end{aligned}
$$

Where and Operational Parameters:
- $E_k$: Unmitigated kinetic impact energy at aerodynamic terminal velocity.
- $m_{\mathrm{system}}$: Total system operational mass.
- $v_{\mathrm{terminal}}$: Unpowered aerodynamic terminal fall velocity.

   - **Kinetic Impact Limit Threshold ($E_k \le E_{\mathrm{threshold}}$):** Statutory regulations establish energy thresholds separating open low-risk operations from certified Specific Category operations. When unmitigated kinetic impact energy exceeds the regulatory exemption threshold ($E_k > E_{\mathrm{threshold}}$), the operation is governed under **Specific Category** mandates requiring certified mitigations (M1–M3) and autonomous containment.

### 5.3 Air Risk Class (ARC) & Strategic Airspace Deconfliction
- **Initial Air Risk Class:** **ARC-b** (Atypical / Segregated Airspace below $h_{\mathrm{operating\_ceiling}}$ in uncontrolled airspace).
- **Strategic Airspace Mitigations:**
  1. Mandatory flight plan filing with U-space / UTM Service Provider prior to launch.
  2. Electronic Conspicuity: Continuous ADS-B Out and direct Remote ID beacon broadcasting vehicle position, velocity, and status at standard broadcast rates.
  3. Tactical Airspace Surveillance: Onboard DAA sensors and visual observer network maintaining continuous 360° situational awareness.

### 5.4 SORA Mitigations (M1–M3)

| Mitigation Level | SORA Mitigation Category | Implementation Mechanism & System Architecture | Target Integrity Level | GRC / ARC Credit |
| :--- | :--- | :--- | :--- | :--- |
| **M1** | Strategic Mitigations for Ground Risk | Controlled ground perimeter fencing, active access control, and operational scheduling during low-occupancy windows. | Medium Integrity (Declared & Audited) | -1 GRC Reduction Credit |
| **M2** | Effects of Ground Impact (Containment) | Redundant autonomous emergency recovery system deploying in t_deploy <= tau_deploy_max; reduces descent velocity to v_impact <= v_safe_limit and kinetic energy to E_impact <= E_threshold. | High Integrity (DO-178C / DO-254 Watchdog) | -1 GRC Reduction Credit |
| **M3** | Emergency Response Plan (ERP) | Formal ERP detailing direct coordination with emergency services, automated emergency beacon broadcast, and hazardous material isolation. | Medium Integrity (Validated by Drill) | Mandatory Prerequisite for Final SORA Authorization |

### 5.5 Containment Margins & Dynamic Geofence Buffers
To guarantee zero-breach containment of the operational volume:
- **Soft Warning Boundary:** Positioned $d_{\mathrm{warning\_buffer}}$ inboard of the primary operational boundary. Reaching this threshold triggers an automated flight path correction and visual/acoustic alert on the GCS.
- **Hard Geofence Boundary:** The outer edge of the contingency volume. Crossing this threshold activates trigger `EMG-05`, initiating an immediate autonomous $180^\circ$ maximum-rate coordinated turnaround maneuver.
- **Buffer Retention Margin:** The Ground Risk Buffer ($R_{\mathrm{GRB}}$) guarantees that in the event of unrecoverable propulsion or control loss at maximum operating ceiling ($h_{\mathrm{ceiling}}$) under maximum wind conditions ($v_{\mathrm{wind\_limit}}$), all system debris comes to rest strictly within the declared buffer zone.
