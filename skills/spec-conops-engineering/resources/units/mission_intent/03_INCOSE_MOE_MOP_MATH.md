| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: INCOSE MoE & MoP Metrics |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics

In accordance with the INCOSE Systems Engineering Handbook v5.0 (§3.2, §3.3) and OMG UAF v2.0, operational objectives and technical performance parameters are quantitatively characterized through formal Measures of Effectiveness (MoE) and Measures of Performance (MoP).

### 3.1 Mathematical Formulations

#### 3.1.1 Operational Availability ($A_o$)
Operational Availability represents the probability that the system operates satisfactorily at any given point in time under stated conditions in an actual operational environment:

$$
\begin{aligned}
A_o &= \frac{\mathrm{MTBM}}{\mathrm{MTBM} + \mathrm{MDT}} \\
\mathrm{MDT} &= \mathrm{MTTR} + \mathrm{MLDT} + \mathrm{MADT}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Operational Availability | A_o | Dimensionless | A_o >= A_o_threshold | Operational availability probability |
| Mean Time Between Maintenance | MTBM | hr | MTBM >= MTBM_min | Mean operating interval between scheduled/unscheduled maintenance |
| Mean Down Time | MDT | hr | MDT <= MDT_max | Total non-operational outage duration |
| Mean Time to Repair | MTTR | hr | MTTR <= MTTR_max | Active corrective maintenance time |
| Mean Logistics Delay Time | MLDT | hr | MLDT <= MLDT_max | Spare parts and support supply chain latency |
| Mean Administrative Delay Time | MADT | hr | MADT <= MADT_max | Operational sign-off and administrative overhead |

#### 3.1.2 State Estimation Error Covariance Matrix ($\mathbf{P}_{\mathrm{state}}$)
The precision of target state tracking and internal state estimation is governed by the discrete-time Kalman filter covariance recursion:

$$
\begin{aligned}
\mathbf{P}_{k|k-1} &= \mathbf{F}_k \mathbf{P}_{k-1|k-1} \mathbf{F}_k^\top + \mathbf{Q}_k \\
\mathbf{K}_k &= \mathbf{P}_{k|k-1} \mathbf{H}_k^\top \left( \mathbf{H}_k \mathbf{P}_{k|k-1} \mathbf{H}_k^\top + \mathbf{R}_k \right)^{-1} \\
\mathbf{P}_{k|k} &= \left( \mathbf{I} - \mathbf{K}_k \mathbf{H}_k \right) \mathbf{P}_{k|k-1} \left( \mathbf{I} - \mathbf{K}_k \mathbf{H}_k \right)^\top + \mathbf{K}_k \mathbf{R}_k \mathbf{K}_k^\top
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| A Priori State Covariance | P_k\|k-1 | m^2 | P_k\|k-1 > 0 | Predicted state estimate error covariance matrix |
| State Transition Matrix | F_k | Dimensionless | Deterministic | Linearized kinematics state transition model |
| Process Noise Covariance | Q_k | m^2/s^2 | Q_k >= 0 | System and disturbance stochastic process noise |
| Kalman Gain Matrix | K_k | Dimensionless | Optimal Gain | Minimum-mean-square-error weighting matrix |
| Observation Matrix | H_k | Dimensionless | Measurement Map | Sensor observation geometry transformation matrix |
| Measurement Noise Covariance | R_k | m^2 | R_k > 0 | Sensor measurement uncertainty matrix |
| A Posteriori State Covariance | P_k\|k | m^2 | norm(P_state) <= norm_P_threshold | Updated state error covariance |

#### 3.1.3 Communication Latency ($\tau_{\mathrm{comm}}$) and Packet Delivery Ratio ($\mathrm{PDR}$)
The end-to-end command-and-control communication latency is bounded by the summation of discrete serialization, transmission, propagation, and decoding delays:

$$
\begin{aligned}
\tau_{\mathrm{comm}} &= \tau_{\mathrm{encode}} + \tau_{\mathrm{tx}} + \tau_{\mathrm{prop}} + \tau_{\mathrm{rx}} + \tau_{\mathrm{decode}} \le \tau_{\mathrm{bound}} \\
\mathrm{PDR} &= \frac{N_{\mathrm{received}}}{N_{\mathrm{transmitted}}} \ge \mathrm{PDR}_{\mathrm{threshold}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Total Communication Latency | tau_comm | ms | tau_comm <= tau_comm_threshold | Total end-to-end communication latency bound |
| Telemetry Encoding Delay | tau_encode | ms | tau_encode <= tau_encode_max | Message serialization and encryption latency |
| Physical Transmission Delay | tau_tx | ms | tau_tx <= tau_tx_max | Physical layer framing and modulation delay |
| Propagation Delay | tau_prop | ms | tau_prop = Range / c | Speed-of-light propagation over path |
| Reception & Buffer Delay | tau_rx | ms | tau_rx <= tau_rx_max | Demodulation, sync, and buffer transfer |
| Telemetry Decoding Delay | tau_decode | ms | tau_decode <= tau_decode_max | Decryption, validation, and parser ingestion |
| Packet Delivery Ratio | PDR | Dimensionless | PDR >= PDR_threshold | Ratio of successfully received to transmitted packets |

#### 3.1.4 Throughput Capacity ($\dot{C}_{\mathrm{capacity}}$) and State Coverage Ratio ($C_{\mathrm{coverage}}$)
Data processing throughput capacity and state space coverage over the operational domain:

$$
\begin{aligned}
\dot{C}_{\mathrm{capacity}} &= \frac{\Delta \mathrm{Data}}{\Delta t} \ge \dot{C}_{\mathrm{min}} \\
C_{\mathrm{coverage}} &= \frac{\Omega_{\mathrm{surveyed}}}{\Omega_{\mathrm{total}}} \ge C_{\mathrm{threshold}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:

| Parameter | Symbol | Units | Constraint / Rule | Description |
| :--- | :--- | :--- | :--- | :--- |
| Instantaneous Throughput Capacity | C_dot_capacity | MB/s | C_dot_capacity >= C_dot_min | Instantaneous processed data per unit time |
| Processed Data Volume | Delta_Data | MB | Delta_Data > 0 | Discrete payload data batch volume |
| Sampling Time Interval | Delta_t | s | Delta_t > 0 | Operational measurement time step |
| Surveyed State Volume | Omega_surveyed | m^3 | Omega_surveyed <= Omega_total | Successfully observed operational state space |
| Total State Volume | Omega_total | m^3 | Omega_total > 0 | Total designated operational state volume |
| State Coverage Ratio | C_coverage | Dimensionless | C_coverage >= C_threshold | Ratio of surveyed state volume to total mission volume |

---

### 3.2 INCOSE SEH v5.0 Metrics Summary Table

| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | Operational Availability | Ao = MTBM / (MTBM + MDT) | A_o_threshold | A_o_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-02 | MoE | State Space Coverage Ratio | C_coverage = Omega_surveyed / Omega_total | C_coverage_threshold | C_coverage_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |
| MoE-03 | MoE | State Identification Probability | P_ID = N_correct / N_targets | P_ID_threshold | P_ID_objective | Dimensionless | INCOSE SEH v5.0 §3.3 |
| MoP-01 | MoP | Trajectory Tracking Deviation | max norm(p_act - p_cmd)_2D | epsilon_xtrack_threshold | epsilon_xtrack_objective | m | IEEE Std 1558-2020 §4.1 |
| MoP-02 | MoP | Communication Transport Latency | tau_comm = tau_enc + tau_tx + tau_prop + tau_rx + tau_dec | tau_comm_threshold | tau_comm_objective | ms | IEEE Std 1558-2020 §4.5 |
| MoP-03 | MoP | Navigation State Covariance Norm | norm(P_state)_2D | norm_P_threshold | norm_P_objective | m | IEEE Std 1558-2020 §4.2 |
| MoP-04 | MoP | Packet Delivery Ratio | PDR = N_received / N_transmitted | PDR_threshold | PDR_objective | Dimensionless | MIL-STD-188-220E §5.3 |
| MoP-05 | MoP | State Location Error Bound | Error_state = 0.589 * (sigma_x + sigma_y) | Error_threshold | Error_objective | m | INCOSE SEH v5.0 §3.3 |
| MoP-06 | MoP | Resource Statutory Reserve Ratio | Ratio = R_reserve / R_capacity | Ratio_reserve_threshold | Ratio_reserve_objective | Dimensionless | INCOSE SEH v5.0 §3.2 |
