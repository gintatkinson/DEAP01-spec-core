| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: INCOSE MoE & MoP Metrics |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics

In accordance with the INCOSE Systems Engineering Handbook v5.0 (§3.2, §3.3) and NATO STANAG 4586, operational objectives and technical performance parameters are quantitatively characterized through formal Measures of Effectiveness (MoE) and Measures of Performance (MoP).

### 3.1 Mathematical Formulations

#### 3.1.1 Operational Availability ($A_o$)
Operational Availability represents the probability that the system operates satisfactorily at any given point in time under stated conditions in an actual operational environment:

$$
\begin{aligned}
A_o &= \frac{\mathrm{MTBM}}{\mathrm{MTBM} + \mathrm{MDT}} \\
\mathrm{MDT} &= \mathrm{MTTR} + \mathrm{MLDT} + \mathrm{MADT}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Nominal Value | Units | Constraint / Rule |
| :--- | :--- | :--- | :--- | :--- |
| Operational Availability | A_o | 0.985 | Dimensionless | A_o >= 0.950 (Threshold), A_o >= 0.990 (Objective) |
| Mean Time Between Maintenance | MTBM | 120.0 | hr | Mean operating interval between scheduled/unscheduled maintenance |
| Mean Down Time | MDT | 1.8 | hr | Total non-operational outage duration |
| Mean Time to Repair | MTTR | 0.5 | hr | Active corrective maintenance time |
| Mean Logistics Delay Time | MLDT | 0.8 | hr | Spare parts and ground support supply chain latency |
| Mean Administrative Delay Time | MADT | 0.5 | hr | Operational sign-off and pre-flight administrative overhead |

#### 3.1.2 Kalman Filter Tracking Error Covariance Matrix ($\mathbf{P}_{\text{cov}}$)
The precision of target tracking and navigation state estimation is governed by the discrete-time Extended Kalman Filter (EKF) covariance recursion:

$$
\begin{aligned}
\mathbf{P}_{k|k-1} &= \mathbf{F}_k \mathbf{P}_{k-1|k-1} \mathbf{F}_k^T + \mathbf{Q}_k \\
\mathbf{K}_k &= \mathbf{P}_{k|k-1} \mathbf{H}_k^T \left( \mathbf{H}_k \mathbf{P}_{k|k-1} \mathbf{H}_k^T + \mathbf{R}_k \right)^{-1} \\
\mathbf{P}_{k|k} &= \left( \mathbf{I} - \mathbf{K}_k \mathbf{H}_k \right) \mathbf{P}_{k|k-1} \left( \mathbf{I} - \mathbf{K}_k \mathbf{H}_k \right)^T + \mathbf{K}_k \mathbf{R}_k \mathbf{K}_k^T
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Nominal Value | Units | Constraint / Rule |
| :--- | :--- | :--- | :--- | :--- |
| A Priori State Covariance | P_k\|k-1 | 0.25 | m^2 | Predicted state estimate error covariance matrix |
| State Transition Matrix | F_k | Dimensionless | Dimensionless | Linearized kinematics state transition model |
| Process Noise Covariance | Q_k | 0.01 | m^2/s^2 | Accelerometer and gyroscope stochastic process noise |
| Kalman Gain Matrix | K_k | Dimensionless | Dimensionless | Optimal minimum-mean-square-error weighting matrix |
| Observation Matrix | H_k | Dimensionless | Dimensionless | Sensor observation geometry transformation matrix |
| Measurement Noise Covariance | R_k | 0.04 | m^2 | GNSS / RTK and optical sensor measurement uncertainty |
| A Posteriori State Covariance | P_k\|k | 0.09 | m^2 | Updated state error covariance; norm(P_cov) <= 3.0 m (Threshold) |

#### 3.1.3 C2 Transport Latency ($\tau_{\text{C2}}$) and Packet Delivery Ratio ($\mathrm{PDR}$)
The end-to-end command-and-control communication latency is bounded by the summation of discrete serialization, propagation, and parsing delays:

$$
\begin{aligned}
\tau_{\text{C2}} &= \tau_{\text{encode}} + \tau_{\text{tx}} + \tau_{\text{prop}} + \tau_{\text{rx}} + \tau_{\text{decode}} \le \tau_{\text{bound}} \\
\mathrm{PDR} &= \frac{N_{\mathrm{received}}}{N_{\mathrm{transmitted}}} \ge \mathrm{PDR}_{\mathrm{threshold}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Nominal Value | Units | Constraint / Rule |
| :--- | :--- | :--- | :--- | :--- |
| Total C2 Transport Latency | tau_C2 | 18.5 | ms | tau_C2 <= 50.0 ms (Threshold), tau_C2 <= 10.0 ms (Objective) |
| Telemetry Encoding Delay | tau_encode | 2.5 | ms | Message serialization and AES-256-GCM encryption latency |
| RF Transmission Delay | tau_tx | 4.0 | ms | Physical layer packet framing and modulation delay |
| Air-to-Ground Propagation Delay | tau_prop | 0.05 | ms | Speed-of-light propagation over 15.0 km line-of-sight path |
| RF Reception & Buffer Delay | tau_rx | 5.5 | ms | Demodulation, preamble sync, and DMA buffer transfer |
| Telemetry Decoding Delay | tau_decode | 6.45 | ms | Decryption, integrity hash validation, and AST ingestion |
| Packet Delivery Ratio | PDR | 0.998 | Dimensionless | PDR >= 0.980 (Threshold), PDR >= 0.999 (Objective) |

#### 3.1.4 Mission Area Coverage Rate ($\dot{A}_{\text{cov}}$) and Area Coverage Ratio ($C_{\text{area}}$)
Sensor payload geometric sweeping dynamics over a planar surveillance footprint:

$$
\begin{aligned}
\dot{A}_{\text{cov}} &= 2 \cdot h_{\mathrm{AGL}} \cdot \tan\left(\frac{\theta_{\mathrm{FOV}}}{2}\right) \cdot v_{\mathrm{ground}} \cdot (1 - \eta_{\mathrm{overlap}}) \\
C_{\mathrm{area}} &= \frac{A_{\mathrm{surveyed}}}{A_{\mathrm{total}}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Nominal Value | Units | Constraint / Rule |
| :--- | :--- | :--- | :--- | :--- |
| Instantaneous Coverage Rate | A_dot_cov | 0.45 | km^2/min | Instantaneous mapped area per unit time |
| Altitude Above Ground Level | h_AGL | 120.0 | m | Operating ceiling above target terrain |
| Payload Field of View Angle | theta_FOV | 60.0 | deg | Optical / thermal sensor angular aperture |
| Ground Speed Velocity | v_ground | 20.0 | m/s | Nominal cruising ground track velocity |
| Swath Overlap Efficiency Factor | eta_overlap | 0.20 | Dimensionless | Overlap ratio to prevent blind spots (20% lateral overlap) |
| Area Coverage Ratio | C_area | 0.975 | Dimensionless | C_area >= 0.900 (Threshold), C_area >= 0.990 (Objective) |

---

### 3.2 INCOSE SEH v5.0 Metrics Summary Table

| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Operational Availability | Ao = MTBM / (MTBM + MDT) | 0.950 | 0.990 | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-02 | MoE | Mission Area Coverage Ratio | C_area = A_surveyed / A_total | 0.900 | 0.990 | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-03 | MoE | Target Identification Probability | P_ID = N_correct / N_targets | 0.920 | 0.980 | Dimensionless | NATO STANAG 4586 Annex B §3.5 |
| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | 5.0 | 1.0 | m | RTCA DO-365B §2.2.3 |
| MoP-02 | MoP | C2 Telemetry Transport Latency | tau_C2 = tau_enc + tau_tx + tau_prop + tau_rx + tau_dec | 50.0 | 10.0 | ms | NATO STANAG 4586 §4.5 |
| MoP-03 | MoP | Navigation State Covariance Norm | norm(P_cov)_2D | 3.0 | 0.5 | m | IEEE Std 1558-2020 §4.2 |
| MoP-04 | MoP | RF Packet Delivery Ratio | PDR = N_received / N_transmitted | 0.980 | 0.999 | Dimensionless | MIL-STD-188-220E §5.3 |
| MoP-05 | MoP | Target Location Error (CEP90) | TLE_CEP90 = 0.589 * (sigma_x + sigma_y) | 5.0 | 1.0 | m | NATO STANAG 4586 Annex B §3.5 |
| MoP-06 | MoP | Bingo Energy Statutory Reserve Ratio | Ratio = E_reserve / E_capacity | 0.200 | 0.250 | Dimensionless | JARUS SORA v2.5 Annex E §2.1 |
