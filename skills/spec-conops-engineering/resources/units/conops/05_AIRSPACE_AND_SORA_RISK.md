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
R_{\mathrm{GRB}} &= h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}}) + v_{\mathrm{wind,max}} \cdot \sqrt{\frac{2 h_{\mathrm{max}}}{g}} + d_{\mathrm{glide,max}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Value | Units | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max Altitude / Ceiling | h_max | 120.0 | m | Maximum operating ceiling above reference ground surface |
| Impact Angle | theta_impact | 45.0 | deg | SORA 1:1 rule worst-case ballistic impact trajectory angle |
| Max Operational Wind Speed | v_wind_max | 15.0 | m/s | Maximum operational wind speed limit (MIL-STD-810H) |
| Gravitational Acceleration | g | 9.80665 | m/s^2 | Standard terrestrial gravitational acceleration constant |
| Maximum Glide Distance | d_glide_max | 50.0 | m | Maximum unpowered lateral displacement margin |
| Ground Risk Buffer Radius | R_GRB | 200.0 | m | Declared ground risk buffer lateral containment radius |
| Terminal Fall Velocity | v_terminal | 25.0 | m/s | Estimated unpowered descent aerodynamic terminal velocity |
| Impact Kinetic Energy | E_impact | 1562.5 | J | Kinetic energy at operational boundary impact (m = 5.0 kg) |
| Ground Risk Classification | GRC | GRC-3 | Dimensionless | JARUS SORA v2.5 intrinsic ground risk class rating |

Given $h_{\mathrm{max}} = 120.0\text{ m}$, $\theta_{\mathrm{impact}} = 45.0^\circ$ ($\tan(45^\circ) = 1.0$), and $v_{\mathrm{wind,max}} = 15.0\text{ m/s}$:
- Ballistic fall time: $t_{\mathrm{fall}} = \sqrt{\frac{2 \cdot 120.0}{9.80665}} \approx 4.947\text{ s}$
- Wind drift distance: $d_{\mathrm{drift}} = 15.0 \cdot 4.947 \approx 74.20\text{ m}$
- Ballistic trajectory distance: $d_{\mathrm{ballistic}} = 120.0 \cdot 1.0 = 120.0\text{ m}$
- Theoretical minimum buffer: $R_{\mathrm{min}} = 120.0 + 74.20 = 194.20\text{ m}$
- Declared $R_{\mathrm{GRB}} = 200.0\text{ m}$ exceeds the theoretical minimum ($200.0\text{ m} \ge 194.20\text{ m}$), providing a validated safety margin.

### 5.2 JARUS SORA v2.5 Ground Risk Class (GRC) & Kinetic Impact Limits
1. **Intrinsic Ground Risk Class (Initial GRC):**
   - Maximum characteristic dimension: $L_{\mathrm{char}} = 1.8\text{ m}$.
   - Typical operational speed: $V_{\mathrm{cruise}} = 22.0\text{ m/s}$.
   - Operational scenario: Over controlled ground area with sparse adjacent population density (< 5 persons / km²).
   - Intrinsic Ground Risk Class: **GRC-3** per SORA v2.5 Table 2.

2. **Kinetic Impact Energy Analysis ($E_k$):**
   - Unmitigated kinetic energy formulation at aerodynamic terminal velocity:
$$
\begin{aligned}
E_k &= \frac{1}{2} m v^2
\end{aligned}
$$
   - Evaluating for vehicle mass $m = 5.0\text{ kg}$ and unpowered terminal velocity $v = 25.0\text{ m/s}$ yields an unmitigated kinetic impact energy of $E_k = 1562.5\text{ J}$.
   - **Kinetic Impact Limit Threshold ($E_k \le 34\text{ J}$):** Statutory regulations (e.g., EASA Open Category A1 / FAA Category 1) exempt operations from complex safety cases only when impact energy is strictly below $34\text{ J}$ (the human skull fracture threshold). Because the unmitigated impact energy ($1562.5\text{ J}$) exceeds $34\text{ J}$, the operation is classified as **Specific Category**, requiring mandatory certified mitigations (M1–M3) and autonomous parachute containment to reduce ground impact severity.

### 5.3 Air Risk Class (ARC) & Strategic Airspace Deconfliction
- **Initial Air Risk Class:** **ARC-b** (Atypical / Segregated Airspace below 120 m AGL in uncontrolled Class G airspace).
- **Strategic Airspace Mitigations:**
  1. Mandatory flight plan filing with U-space Service Provider (USSP) at least 60 minutes prior to launch.
  2. Electronic Conspicuity: Continuous ADS-B Out (1090 MHz / 978 MHz UAT) and direct Remote ID (ASTM F3411-22a Bluetooth 5.0 & Wi-Fi Beacon) broadcasting vehicle position at 1.0 Hz.
  3. Tactical Airspace Surveillance: Onboard ADS-B In receiver and visual observer network maintaining continuous 360° situational awareness.

### 5.4 SORA Mitigations (M1–M3)

| Mitigation Level | SORA Mitigation Category | Implementation Mechanism & System Architecture | Target Integrity Level | GRC / ARC Credit |
| :--- | :--- | :--- | :--- | :--- |
| **M1** | Strategic Mitigations for Ground Risk | Controlled ground perimeter fencing, active perimeter access control, and operational scheduling during non-working hours. | Medium Integrity (Declared & Audited) | -1 GRC Reduction (GRC-3 -> GRC-2) |
| **M2** | Effects of Ground Impact (Containment & Parachute) | Redundant autonomous ballistic parachute recovery system deploying in < 20 ms upon tumbling detection; reduces descent velocity to < 3.5 m/s and impact energy to < 30.6 J (< 34 J limit). | High Integrity (DO-178C / DO-254 Hardware Watchdog) | -1 GRC Reduction (GRC-2 -> GRC-1) |
| **M3** | Emergency Response Plan (ERP) | Formal ERP document detailing direct hotlines to local emergency services, automated GPS crash beacon broadcasts, and toxic material isolation procedures. | Medium Integrity (Validated by Drill) | Mandatory Prerequisite for Final SORA Authorization |

### 5.5 Containment Margins & Dynamic Geofence Buffers
To guarantee zero-breach containment of the operational volume:
- **Soft Warning Boundary:** Positioned $50.0\text{ m}$ inboard of the primary operational boundary. Reaching this threshold triggers an automated flight path correction and visual/acoustic alert on the GCS.
- **Hard Geofence Boundary:** The outer edge of the contingency volume. Crossing this threshold activates trigger `EMG-05`, initiating an immediate autonomous $180^\circ$ banking turnaround maneuver or failsafe hover.
- **Buffer Retention Margin:** The $200.0\text{ m}$ Ground Risk Buffer guarantees that even in the event of complete simultaneous propulsion and control failure at maximum operating ceiling ($120.0\text{ m}$) under maximum wind ($15.0\text{ m/s}$), all physical wreckage comes to rest strictly within the declared buffer zone.
