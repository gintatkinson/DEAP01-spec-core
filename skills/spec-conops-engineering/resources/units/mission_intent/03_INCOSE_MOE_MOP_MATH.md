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

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Operational Availability | A_o | Dimensionless | A_o >= A_o_threshold | Operational availability probability |
| Mean Time Between Maintenance | MTBM | hr | MTBM >= MTBM_min | Mean operating interval between scheduled/unscheduled maintenance |
| Mean Down Time | MDT | hr | MDT <= MDT_max | Total non-operational outage duration |
| Mean Time to Repair | MTTR | hr | MTTR <= MTTR_max | Active corrective maintenance time |
| Mean Logistics Delay Time | MLDT | hr | MLDT <= MLDT_max | Spare parts and ground support supply chain latency |
| Mean Administrative Delay Time | MADT | hr | MADT <= MADT_max | Operational sign-off and administrative overhead |

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

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| A Priori State Covariance | P_k\|k-1 | m^2 | P_k\|k-1 > 0 | Predicted state estimate error covariance matrix |
| State Transition Matrix | F_k | Dimensionless | Deterministic | Linearized kinematics state transition model |
| Process Noise Covariance | Q_k | m^2/s^2 | Q_k >= 0 | Sensor and disturbance stochastic process noise |
| Kalman Gain Matrix | K_k | Dimensionless | Optimal Gain | Minimum-mean-square-error weighting matrix |
| Observation Matrix | H_k | Dimensionless | Measurement Map | Sensor observation geometry transformation matrix |
| Measurement Noise Covariance | R_k | m^2 | R_k > 0 | Sensor measurement uncertainty matrix |
| A Posteriori State Covariance | P_k\|k | m^2 | norm(P_cov) <= norm_P_threshold | Updated state error covariance |

#### 3.1.3 C2 Transport Latency ($\tau_{\text{C2}}$) and Packet Delivery Ratio ($\mathrm{PDR}$)
The end-to-end command-and-control communication latency is bounded by the summation of discrete serialization, propagation, and parsing delays:

$$
\begin{aligned}
\tau_{\text{C2}} &= \tau_{\text{encode}} + \tau_{\text{tx}} + \tau_{\text{prop}} + \tau_{\text{rx}} + \tau_{\text{decode}} \le \tau_{\text{bound}} \\
\mathrm{PDR} &= \frac{N_{\mathrm{received}}}{N_{\mathrm{transmitted}}} \ge \mathrm{PDR}_{\mathrm{threshold}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Total C2 Transport Latency | tau_C2 | ms | tau_C2 <= tau_C2_threshold | Total end-to-end communication latency bound |
| Telemetry Encoding Delay | tau_encode | ms | tau_encode <= tau_encode_max | Message serialization and encryption latency |
| RF Transmission Delay | tau_tx | ms | tau_tx <= tau_tx_max | Physical layer packet framing and modulation delay |
| Propagation Delay | tau_prop | ms | tau_prop = Range / c | Speed-of-light propagation over RF path |
| Reception & Buffer Delay | tau_rx | ms | tau_rx <= tau_rx_max | Demodulation, preamble sync, and DMA buffer transfer |
| Telemetry Decoding Delay | tau_decode | ms | tau_decode <= tau_decode_max | Decryption, integrity validation, and parser ingestion |
| Packet Delivery Ratio | PDR | Dimensionless | PDR >= PDR_threshold | Ratio of successfully received to transmitted packets |

#### 3.1.4 Mission Area Coverage Rate ($\dot{A}_{\text{cov}}$) and Area Coverage Ratio ($C_{\text{area}}$)
Sensor payload geometric sweeping dynamics over a planar surveillance footprint:

$$
\begin{aligned}
\dot{A}_{\text{cov}} &= 2 \cdot h_{\mathrm{AGL}} \cdot \tan\left(\frac{\theta_{\mathrm{FOV}}}{2}\right) \cdot v_{\mathrm{ground}} \cdot (1 - \eta_{\mathrm{overlap}}) \\
C_{\mathrm{area}} &= \frac{A_{\mathrm{surveyed}}}{A_{\mathrm{total}}}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Instantaneous Coverage Rate | A_dot_cov | km^2/min | A_dot_cov >= A_dot_cov_min | Instantaneous mapped area per unit time |
| Altitude Above Ground Level | h_AGL | m | h_min <= h_AGL <= h_ceiling | Operating altitude above target terrain |
| Payload Field of View Angle | theta_FOV | deg | 0 < theta_FOV < 180 | Sensor angular aperture |
| Ground Speed Velocity | v_ground | m/s | v_min <= v_ground <= v_max | Operational cruising ground track velocity |
| Swath Overlap Factor | eta_overlap | Dimensionless | 0 <= eta_overlap < 1 | Overlap ratio to prevent blind spots |
| Area Coverage Ratio | C_area | Dimensionless | C_area >= C_area_threshold | Ratio of surveyed area to total mission area |

---

### 3.2 INCOSE SEH v5.0 Metrics Summary Table

| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Operational Availability | Ao = MTBM / (MTBM + MDT) | Ao_threshold | Ao_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-02 | MoE | Mission Area Coverage Ratio | C_area = A_surveyed / A_total | C_area_threshold | C_area_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-03 | MoE | Target Identification Probability | P_ID = N_correct / N_targets | P_ID_threshold | P_ID_objective | Dimensionless | NATO STANAG 4586 Annex B §3.5 |
| MoP-01 | MoP | Cross-Track Waypoint Deviation | max norm(p_act - p_cmd)_2D | epsilon_xtrack_threshold | epsilon_xtrack_objective | m | RTCA DO-365B §2.2.3 |
| MoP-02 | MoP | C2 Telemetry Transport Latency | tau_C2 = tau_enc + tau_tx + tau_prop + tau_rx + tau_dec | tau_C2_threshold | tau_C2_objective | ms | NATO STANAG 4586 §4.5 |
| MoP-03 | MoP | Navigation State Covariance Norm | norm(P_cov)_2D | norm_P_threshold | norm_P_objective | m | IEEE Std 1558-2020 §4.2 |
| MoP-04 | MoP | RF Packet Delivery Ratio | PDR = N_received / N_transmitted | PDR_threshold | PDR_objective | Dimensionless | MIL-STD-188-220E §5.3 |
| MoP-05 | MoP | Target Location Error (CEP90) | TLE_CEP90 = 0.589 * (sigma_x + sigma_y) | TLE_threshold | TLE_objective | m | NATO STANAG 4586 Annex B §3.5 |
| MoP-06 | MoP | Bingo Energy Statutory Reserve Ratio | Ratio = E_reserve / E_capacity | Ratio_reserve_threshold | Ratio_reserve_objective | Dimensionless | JARUS SORA v2.5 Annex E §2.1 |
