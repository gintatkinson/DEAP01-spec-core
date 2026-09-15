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

| Parameter | Symbol | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max State Coordinate Limit | x_max | m | x_max <= x_operational_max | Maximum upper boundary limit in operational state space |
| Maximum Operational Velocity | v_max | m/s | v_max <= v_envelope_max | Maximum operational velocity in state space |
| Containment Response Time | t_resp | s | t_resp <= tau_containment_req | Maximum duration from anomaly detection to actuator containment execution |
| Deceleration / Dissipation Rate | a_decel | m/s^2 | a_decel >= a_decel_min | Minimum deceleration capability under active braking / energy dissipation |
| Containment Buffer Margin | d_margin | m | d_margin >= d_margin_min | Margin accounting for state estimation uncertainty and dynamic disturbances |
| Containment Buffer Radius | R_buffer | m | R_buffer >= R_buffer_min | Declared containment buffer radius |
| System Total Mass | m_system | kg | m_system <= m_system_max | Total operational mass of the cyber-physical system |

### 5.1.1 Environmental Disturbance & State Displacement Sensitivity Analysis
The containment buffer ($R_{\mathrm{buffer}}$) must account for worst-case uncommanded trajectory drift induced by environmental disturbances (e.g., wind, fluid currents, thermal gradients) acting over the duration of the containment reaction sequence:

$$
\begin{aligned}
d_{\mathrm{disturbance}} &= v_{\mathrm{disturb}} \cdot t_{\mathrm{reaction}} \\
d_{\mathrm{impact}} &= d_{\mathrm{nominal}} + d_{\mathrm{disturbance}} + d_{\mathrm{glide}} \\
\Delta R &= R_{\mathrm{buffer}} - d_{\mathrm{impact}}
\end{aligned}
$$

| Parameter | Symbol | Units | Description |
| :--- | :--- | :--- | :--- |
| Disturbance Velocity | v_disturb | m/s | Maximum environmental disturbance rate (e.g., crosswind, current) |
| Reaction Time | t_reaction | s | Time from boundary threshold crossing to full containment |
| Disturbance Displacement | d_disturbance | m | State displacement due to environmental forcing |
| Unpowered Coasting Margin | d_glide | m | Maximum unpowered lateral displacement margin |
| Nominal Impact Radius | d_nominal | m | Baseline trajectory offset |
| Total Impact Radius | d_impact | m | Total state space offset including all error margins |
| Declared Buffer Radius | R_buffer | m | Declared containment buffer radius |
| Containment Margin | Delta R | m | Containment safety margin |

### 5.2 Intrinsic Risk Classification & Energy Containment Physics Derivations
1. **Intrinsic Risk Classification (Initial RC):**
   - Maximum characteristic physical dimension: $L_{\mathrm{char}}$.
   - Nominal operational velocity: $v_{\mathrm{nominal}}$.
   - Operational context: Bounded operational domain with segregated non-participant zones.
   - Intrinsic Risk Class: Derived per generic system safety guidelines (e.g., MIL-STD-882E).

2. **Kinetic Energy Physics Derivations:**
   - **Unmitigated Runaway Energy Derivation:**
$$
\begin{aligned}
E_{k,\mathrm{unmitigated}} &= \frac{1}{2} m_{\mathrm{system}} v_{\mathrm{runaway}}^2
\end{aligned}
$$
   - **Mitigated Dynamic Equilibrium Derivation:**
     Under failsafe containment actuation, energy dissipation mechanisms reduce the state velocity to a mitigated equilibrium:
$$
\begin{aligned}
E_{k,\mathrm{mitigated}} &= \frac{1}{2} m_{\mathrm{system}} v_{\mathrm{mitigated}}^2
\end{aligned}
$$

| Parameter | Symbol | Units | Constraint Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| System Mass | m_system | kg | m_system <= m_max | Total system mass |
| Runaway Velocity | v_runaway | m/s | v_runaway <= v_terminal | Maximum unconstrained fault velocity |
| Mitigated Velocity | v_mitigated | m/s | v_mitigated <= v_safe | Equilibrium failsafe velocity |
| Unmitigated Energy | E_k_unmitigated | J | E_k_unmitigated > E_threshold | Total unmitigated impact kinetic energy |
| Mitigated Energy | E_k_mitigated | J | E_k_mitigated <= E_threshold | Failsafe-mitigated impact kinetic energy |
| Regulatory Energy Threshold | E_threshold | J | E_threshold >= E_k_mitigated | Maximum kinetic energy threshold for containment |

### 5.3 System Level Emergency Failsafe Containment Mechanism
To guarantee robust operational safety across all operating states, the system incorporates verified emergency containment mechanisms:
1. **Primary Containment Triggering:**
   - Continuous state space boundary monitoring at high frequency.
   - Independent safety watchdog triggering emergency containment deployment to bound state limits.
2. **Abstract Containment Modalities:**
   - Energy isolation (e.g., power bus disconnect, hydraulic pressure venting).
   - Friction/Drag induction (e.g., mechanical braking, drogue deployment, retro-thrust).
   - State space passivation (e.g., rendering the system inert to prevent subsequent hazard propagation).

### 5.4 Strategic Deconfliction & State Separation
- **Strategic Boundary Mitigations:**
  1. Mandatory operational envelope registry with overarching coordination authority.
  2. Telemetry broadcasting of current state vector and functional health.
  3. Tactical Environmental Surveillance ensuring the assumed disturbance limits (e.g. $v_{\mathrm{disturb}}$) are not exceeded.

### 5.5 Containment Margins & Dynamic Exclusion Buffers
To guarantee zero-breach containment of the operational state space:
- **Soft Warning Boundary:** Positioned inbound of the primary operational boundary. Crossing triggers an automated trajectory correction and operator alert.
- **Hard Containment Boundary:** The outer edge of the nominal envelope. Crossing activates an immediate autonomous maximum-rate boundary reversal or isolation maneuver.
- **Buffer Retention Margin:** The containment buffer ($R_{\mathrm{buffer}}$) guarantees that under unrecoverable actuation loss, all system states remain strictly confined within the declared safety zone.
