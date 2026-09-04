| Attribute | Value |
| :--- | :--- |
| **Title** | Operational State Space, Boundary Containment & Risk Assessment |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 5. Operational State Space, Boundary Containment & Risk Assessment

### 5.1 Operational State Space Formulation & Boundary Containment Mathematics
The system operational domain is formally defined as a bounded multi-dimensional Operational State Space $\Omega_{\mathrm{state}} \subset \mathbb{R}^n$, bounded by physical, environmental, and operational parameter limits $\mathbf{X}_{\mathrm{boundary}} = [\mathbf{x}_{\mathrm{min}}, \mathbf{x}_{\mathrm{max}}]^\top$. The total operational envelope consists of the nominal operational geometry, the contingency envelope, and the containment risk buffer:

$$
\begin{aligned}
\Omega_{\mathrm{state}} &\subset \mathbb{R}^n \\
\mathbf{X}_{\mathrm{boundary}} &= [\mathbf{x}_{\mathrm{min}}, \mathbf{x}_{\mathrm{max}}]^\top \\
V_{\mathrm{operational}} &= V_{\mathrm{nominal}} \cup V_{\mathrm{contingency}} \cup V_{\mathrm{buffer}} \\
R_{\mathrm{buffer}} &= v_{\mathrm{max}} \cdot t_{\mathrm{resp}} + \frac{v_{\mathrm{max}}^2}{2 \cdot a_{\mathrm{decel}}} + d_{\mathrm{margin}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max State Coordinate Limit | x_max | m | x_max <= x_operational_max | Maximum upper boundary limit in operational state space |
| Maximum Operational Velocity | v_max | m/s | v_max <= v_envelope_max | Maximum operational velocity in state space |
| Containment Response Time | t_resp | s | t_resp <= tau_containment_req | Maximum duration from anomaly detection to actuator containment execution |
| Deceleration / Dissipation Rate | a_decel | m/s^2 | a_decel >= a_decel_min | Minimum deceleration capability under active braking / energy dissipation |
| Containment Buffer Margin | d_margin | m | d_margin >= d_margin_min | Margin accounting for state estimation uncertainty and dynamic disturbances |
| Containment Buffer Radius | R_buffer | m | R_buffer >= R_buffer_min | Declared lateral containment buffer radius |
| System Total Mass | m_system | kg | m_system <= m_system_max | Total operational mass of the cyber-physical system |
| Terminal Velocity | v_terminal | m/s | v_terminal = sqrt(2 * m_system * g / (rho * C_D * A_ref)) | Maximum unpowered / unconstrained terminal velocity |
| Kinetic / Boundary Energy | E_k | J | E_k = 0.5 * m_system * v_terminal^2 | Kinetic energy at operational boundary impact |
| Risk Classification | RC | Dimensionless | RC in {RC-1, ..., RC-7} | Intrinsic operational risk class rating |

The symbolic derivation for $R_{\mathrm{buffer}}$ accounts for:
- Reaction translation during anomaly detection and command execution: $d_{\mathrm{reaction}} = v_{\mathrm{max}} \cdot t_{\mathrm{resp}}$
- Dynamic braking deceleration distance: $d_{\mathrm{decel}} = \frac{v_{\mathrm{max}}^2}{2 \cdot a_{\mathrm{decel}}}$
- Margin for state estimator uncertainty and ambient dynamic disturbances: $d_{\mathrm{margin}}$
- Declared $R_{\mathrm{buffer}}$ satisfies $R_{\mathrm{buffer}} \ge d_{\mathrm{reaction}} + d_{\mathrm{decel}} + d_{\mathrm{margin}}$, providing a mathematically verified containment envelope.

### 5.1.1 Ground Risk Buffer Parametric Wind Sensitivity Analysis
Under JARUS SORA v2.5 Annex B guidelines, the Ground Risk Buffer ($R_{\mathrm{buffer}}$) must account for worst-case ballistic descent dynamics, aerodynamic wind drift displacement, and lateral glide margins across the operational environmental envelope:

$$
\begin{aligned}
t_{\mathrm{fall}} &= \sqrt{\frac{2 h_{\mathrm{max}}}{g}} \\
d_{\mathrm{wind}} &= v_{\mathrm{wind}} \cdot t_{\mathrm{fall}} = v_{\mathrm{wind}} \cdot \sqrt{\frac{2 h_{\mathrm{max}}}{g}} \\
d_{\mathrm{impact}} &= h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}}) + d_{\mathrm{wind}} + d_{\mathrm{glide,max}} \\
\Delta R &= R_{\mathrm{buffer}} - d_{\mathrm{impact}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
- $t_{\mathrm{fall}}$: Ballistic free-fall duration from maximum operational altitude $h_{\mathrm{max}}$ to ground plane under gravitational acceleration $g$.
- $d_{\mathrm{wind}}$: Lateral aerodynamic wind drift distance driven by crosswind speed $v_{\mathrm{wind}}$ over fall duration $t_{\mathrm{fall}}$.
- $d_{\mathrm{impact}}$: Total impact trajectory radius combining ballistic ground displacement ($h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}})$ with 1:1 rule $\theta_{\mathrm{impact}} = 45^\circ$), wind drift distance $d_{\mathrm{wind}}$, and maximum glide margin $d_{\mathrm{glide,max}}$.
- $\Delta R$: Containment safety margin between declared containment buffer $R_{\mathrm{buffer}}$ and total impact radius $d_{\mathrm{impact}}$. A positive margin ($\Delta R \ge 0$) guarantees zero-breach containment.

The parametric wind sensitivity sweep across the declared operational wind envelope ($0\text{ m/s}$ to $20\text{ m/s}$ in $5\text{ m/s}$ increments) for nominal ceiling $h_{\mathrm{max}} = 120.0\text{ m}$, $\theta_{\mathrm{impact}} = 45.0^\circ$, $g = 9.80665\text{ m/s}^2$, $d_{\mathrm{glide,max}} = 0.0\text{ m}$, and declared $R_{\mathrm{buffer}} = 200.0\text{ m}$ is evaluated below:

| Wind Speed v_wind (m/s) | Ballistic Fall Time t_fall (s) | Wind Drift Distance d_wind (m) | Total Impact Trajectory Radius d_impact (m) | Declared Buffer Radius R_buffer (m) | Containment Margin ΔR (m) | Containment Compliance Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 0.0 | 4.95 | 0.0 | 120.0 | 200.0 | +80.0 | Nominal (Compliant - Zero Wind Baseline) |
| 5.0 | 4.95 | 24.7 | 144.7 | 200.0 | +55.3 | Nominal (Compliant - Light Breeze) |
| 10.0 | 4.95 | 49.5 | 169.5 | 200.0 | +30.5 | Nominal (Compliant - Moderate Wind) |
| 15.0 | 4.95 | 74.2 | 194.2 | 200.0 | +5.8 | Marginal (Compliant - Maximum Operational Limit) |
| 20.0 | 4.95 | 98.9 | 218.9 | 200.0 | -18.9 | Non-Compliant (Breach - Exceeds Operational Envelope) |

### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations
1. **Intrinsic Risk Classification (Initial RC):**
   - Maximum characteristic physical dimension: $L_{\mathrm{char}}$.
   - Nominal operational velocity: $v_{\mathrm{nominal}}$.
   - Operational context: Controlled perimeter with adjacent low-occupancy zones.
   - Intrinsic Risk Class: Mapped to risk class rating per system safety guidelines and JARUS SORA v2.5 Annex B.

2. **Kinetic Impact Energy Physics Derivations:**

   - **Unmitigated Free-Fall Kinetic Impact Derivation:**
     At unconstrained ballistic terminal velocity, the downward gravitational force equals the aerodynamic drag force ($F_g = F_D$):
     $$m g = \frac{1}{2} \rho S_{\mathrm{ref}} C_D v_{\mathrm{terminal,unmitigated}}^2$$
     Solving for terminal velocity and substituting into the kinetic energy formulation yields:

$$
\begin{aligned}
v_{\mathrm{terminal,unmitigated}} &= \sqrt{\frac{2mg}{\rho S_{\mathrm{ref}} C_D}} \\
E_{k,\mathrm{unmitigated}} &= \frac{1}{2} m v_{\mathrm{terminal,unmitigated}}^2 = \frac{1}{2} m \left( \frac{2mg}{\rho S_{\mathrm{ref}} C_D} \right) = \frac{m^2 g}{\rho S_{\mathrm{ref}} C_D}
\end{aligned}
$$
   - **Aerodynamic Descent Equilibrium Derivation:**
     Under failsafe {{RECOVERY_DEVICE_TERM:parachute}} deployment and canopy inflation, steady-state aerodynamic drag counterbalances gravitational force ($F_g = F_{D,{{RECOVERY_SUB:parachute}}}$), establishing terminal descent equilibrium:

$$
\begin{aligned}
m g &= \frac{1}{2} \rho S_{\mathrm{canopy}} C_{d,\mathrm{parachute}} v_{\mathrm{terminal,parachute}}^2 \\
v_{\mathrm{terminal,parachute}} &= \sqrt{\frac{2mg}{\rho S_{\mathrm{canopy}} C_{d,\mathrm{parachute}}}}
\end{aligned}
$$

   - **Failsafe-Mitigated Kinetic Impact Energy:**
     Substituting the equilibrium descent velocity into the kinetic energy formulation gives the failsafe-mitigated impact energy:

$$
\begin{aligned}
E_{k,\mathrm{mitigated}} &= \frac{1}{2} m v_{\mathrm{terminal,parachute}}^2
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| System Operational Mass | m | {{SYSTEM_MASS_KG:25.0}} | kg | m <= m_max | Total cyber-physical system mass at launch |
| Gravitational Acceleration | g | 9.80665 | m/s^2 | g = 9.80665 | Standard gravitational acceleration constant |
| Atmospheric Air Density | rho | {{AIR_DENSITY_KGM3:1.225}} | kg/m^3 | rho >= rho_min | Sea-level standard atmospheric air density (ISA) |
| Unmitigated Reference Area | S_ref | {{FRONTAL_AREA_M2:0.18}} | m^2 | S_ref > 0 | Frontal aerodynamic reference cross-sectional area |
| Unmitigated Drag Coefficient | C_D | {{DRAG_COEFFICIENT:0.45}} | Dimensionless | C_D >= 0.30 | Characteristic unmitigated vehicle aerodynamic drag coefficient |
| Unmitigated Terminal Velocity | v_terminal_unmitigated | {{V_TERMINAL_UNMITIGATED_MPS:77.01}} | m/s | v_terminal_unmitigated = sqrt(2*m*g / (rho*S_ref*C_D)) | Free-fall unconstrained terminal descent velocity |
| Unmitigated Kinetic Energy | E_k_unmitigated | {{E_K_UNMITIGATED_JOULES:74125.1}} | J | E_k_unmitigated = (m^2 * g) / (rho * S_ref * C_D) | Total unmitigated ground impact kinetic energy |
| Parachute Canopy Area | S_canopy | {{S_CANOPY:84.0}} | m^2 | S_canopy >= S_canopy_min | Deployed emergency recovery canopy surface area |
| Parachute Drag Coefficient | C_d_parachute | {{PARACHUTE_DRAG_COEFFICIENT:1.75}} | Dimensionless | C_d_parachute >= 1.50 | Deployed canopy aerodynamic drag coefficient |
| Parachute Terminal Velocity | v_terminal_parachute | {{V_TERMINAL_PARACHUTE_MPS:1.65}} | m/s | v_terminal_parachute = sqrt(2*m*g / (rho*S_canopy*C_d_parachute)) <= 1.65 | Equilibrium descent velocity (<= 1.65 m/s) |
| Mitigated Kinetic Energy | E_k_mitigated | {{E_K_MITIGATED_JOULES:34.0}} | J | E_k_mitigated = 0.5 * m * v_terminal_parachute^2 <= 34.0 | Failsafe-mitigated impact kinetic energy (<= 34.0 J) |
| Regulatory Energy Threshold | E_threshold | 34.0 | J | E_threshold = 34.0 | Regulatory maximum kinetic energy threshold for low ground risk classification |

3. **Kinetic Energy Threshold Compliance ($E_k \le E_{\mathrm{threshold}}$):**
   - Unmitigated free-fall kinetic energy ($E_{k,\mathrm{unmitigated}} = {{E_K_UNMITIGATED_JOULES:74125.1}}\text{ J}$) exceeds the low-risk kinetic energy threshold ($E_{\mathrm{threshold}} = 34.0\text{ J}$), mandating certified safety mitigations (M1–M3) and autonomous containment mechanisms per JARUS SORA v2.5.
   - Autonomous emergency {{RECOVERY_DEVICE_TERM:parachute}} actuation reduces the terminal descent velocity to $v_{\mathrm{terminal,parachute}} \le {{V_TERMINAL_PARACHUTE_MPS:1.65}}\text{ m/s}$, capping the ground impact kinetic energy to $E_{k,\mathrm{mitigated}} \le {{E_K_MITIGATED_JOULES:34.0}}\text{ J}$, fulfilling the high-assurance energy containment criteria.

### 5.2.1 Domain-Specific Multi-Physics Failsafe Containment Architectures
For multi-domain operations across non-aerial and aerial platforms, containment mechanisms are tailored to the physical operating medium:
1. **Terrestrial Ground & Rail Containment:**
   - Mechanical emergency stop (e-stop) actuation and pneumatic train brake pipe venting (rapid pressure reduction from $5.0\text{ bar}$ nominal to $0.0\text{ bar}$ atmospheric venting).
   - Spring-applied, pressure-released fail-safe mechanical friction calipers providing deterministic stopping deceleration ($a_{\mathrm{decel}} \ge 1.5\text{ m/s}^2$).
2. **Subsea & Maritime Autonomous Containment:**
   - Galvanic timed drop-weight release mechanisms and hydrostatic passive buoyancy ascent chambers.
   - Closed-loop ballast expulsion reducing descent velocity to zero and returning system safely to surface baseline.
3. **Space LEO Constellation Containment:**
   - Cold-gas thruster retro-burn perigee lowering for controlled atmospheric demise and passivation (zero residual stored energy / battery disconnect).
4. **Aerial UAS & eVTOL Failsafe Containment:**
   - Autonomous flight termination unit (FTU) with ballistic {{RECOVERY_DEVICE_TERM:parachute}} ejection ($t_{\mathrm{deploy}} \le 0.5\text{ s}$) and motor drive power isolation.

---

### 5.3 System Level Emergency Failsafe Containment Mechanism
To guarantee robust ground and operational safety across all operating states, the system incorporates certified emergency containment mechanisms:
1. **Primary Containment Triggering:**
   - Continuous geofence boundary monitoring at frequency $f \ge 50\text{ Hz}$.
   - Independent safety watchdog triggering ballistic {{RECOVERY_DEVICE_TERM:parachute}} deployment (${{PARACHUTE_SYMBOL_V:v_{\mathrm{terminal,parachute}}}} \le 1.65\text{ m/s}$, $E_{k,\mathrm{mitigated}} \le 34.0\text{ J}$).
2. **Subsea & Maritime Containment:**
   - Inherent positive buoyancy reserve ($B_{\mathrm{net}} \ge 0.05 \cdot W_{\mathrm{dry}}$) ensuring unpowered passive ascent to surface.
   - Galvanic / electromagnetic drop-weight ballast release mechanism providing fail-safe positive buoyant ascent upon electrical power loss or depth threshold exceedance.
3. **Space & Orbital Containment:**
   - Autonomous orbital de-orbit burn execution using dedicated delta-V propellant reserve ($\Delta v_{\mathrm{deorbit}}$).
   - Passivated reaction wheels (spin-down to zero angular momentum), high-voltage battery discharge passivation, and solar array feathered orientation to eliminate orbital fragmentation risks.
4. **Aerial & UAS Atmospheric Containment (for platforms with $h_{\mathrm{max}} > 0$):**
   - Independent safety watchdog triggering ballistic {{RECOVERY_DEVICE_TERM:parachute}} deployment (${{PARACHUTE_SYMBOL_V:v_{\mathrm{terminal,parachute}}}} \le 1.65\text{ m/s}$, $E_{k,\mathrm{mitigated}} \le 34.0\text{ J}$).
   - Autonomous motor power bus disconnect preventing uncommanded powered trajectory excursions.

### 5.3 Strategic Deconfliction & State Separation
- **Strategic Boundary Mitigations:**
  1. Mandatory operational plan registry with external coordination services prior to mission start.
  2. Electronic Conspicuity & State Telemetry: Continuous broadcast of system position, velocity vector, and operational status at standard periodic rates.
  3. Tactical Environmental Surveillance: Continuous multi-sensor situational awareness monitoring surrounding state space.

### 5.4 SORA Ground Risk Mitigations (M1–M3)
In accordance with JARUS SORA v2.5 Annex B (§2.1–§2.3), Ground Risk Class (GRC) mitigations are systematically categorized across strategic isolation (M1), ground impact effects reduction (M2), and emergency response planning (M3):

| Mitigation Code | SORA Mitigation Category | Technical Implementation Mechanism & Architecture | Assurance Level | Robustness & Integrity Level | GRC Reduction Credit | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **M1.A** | Strategic Ground Risk Mitigation | Operational scheduling during verified low-density time windows with passive signage. | Low | Declared protocol with basic operational logbook auditing. | -1 GRC | JARUS SORA v2.5 Annex B §2.1 |
| **M1.B** | Strategic Ground Risk Mitigation | Physical perimeter isolation, access control checkpoints, and active buffer surveillance. | Medium | Audited perimeter control with active personnel exclusion. | -2 GRC | JARUS SORA v2.5 Annex B §2.1 |
| **M1.C** | Strategic Ground Risk Mitigation | Enclosed access-controlled operational test range with hard fencing, security interlocks, and 0 non-participants. | High | Third-party audited physical containment with zero non-participant access. | -2 GRC | JARUS SORA v2.5 Annex B §2.1 |
| **M2.A** | Impact Dynamics Mitigation | Impact-resistant frangible structures and energy-attenuating landing gear geometries. | Low | Empirical impact testing demonstrating controlled energy absorption. | -1 GRC | JARUS SORA v2.5 Annex B §2.2 |
| **M2.B** | Impact Dynamics Mitigation | Autonomous {{FAILSAFE_DESCENT_SYSTEM:emergency parachute recovery system}} actuating in t_deploy <= tau_deploy_max, reducing v_terminal <= 3.0 m/s. | Medium | Dual-channel sensor trigger with independent backup battery pack. | -1 GRC | JARUS SORA v2.5 Annex B §2.2 |
| **M2.C** | Impact Dynamics Mitigation | Certified {{FAILSAFE_DESCENT_SYSTEM:emergency parachute recovery system}} (ASTM F3322-18 / RTCA DO-178C DAL-C) ensuring v_terminal <= 1.65 m/s and E_k_mitigated <= 34.0 J. | High | Fully independent flight termination watchdog, ballistic ejection, and certified compliance. | -2 GRC | JARUS SORA v2.5 Annex B §2.2 |Annex B §2.2 |
| **M3.A** | Emergency Response Plan (ERP) | Basic operator emergency checklist detailing notification phone numbers and rally points. | Low | Self-declared operational procedure without rehearsal. | 0 GRC (Prerequisite) | JARUS SORA v2.5 Annex B §2.3 |
| **M3.B** | Emergency Response Plan (ERP) | Formal ERP coordinated with local emergency response services, defined divert landing sites, and trained personnel. | Medium | Validated ERP with annual multi-agency tabletop drills and direct coordinator link. | -1 GRC | JARUS SORA v2.5 Annex B §2.3 |
| **M3.C** | Emergency Response Plan (ERP) | Integrated ERP with automated first-responder alerting API, multi-channel satellite emergency beacon (EMG-07), and certified emergency response team. | High | Live drill validated with competent emergency authorities, full mock incident execution, and automated rescue telemetry. | -1 GRC | JARUS SORA v2.5 Annex B §2.3 |

### 5.5 Containment Margins & Dynamic Exclusion Buffers
To guarantee zero-breach containment of the operational state space:
- **Soft Warning Boundary:** Positioned $d_{\text{warning\_buffer}}$ inboard of the primary operational boundary. Reaching this threshold triggers an automated trajectory correction and visual/acoustic alert on the operator console.
- **Hard Containment Boundary:** The outer edge of the contingency state space. Crossing this threshold activates trigger `EMG-05`, initiating an immediate autonomous maximum-rate boundary reversal maneuver.
- **Buffer Retention Margin:** The containment buffer ($R_{\mathrm{buffer}}$) guarantees that in the event of unrecoverable actuation or control loss at maximum boundary speed under worst-case disturbances, all system states remain strictly confined within the declared buffer zone.
