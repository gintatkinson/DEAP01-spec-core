| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: PACE C2 Communications Plan |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 5. PACE C2 Link Communications Plan

In accordance with NATO STANAG 4586 (§4.5) and RTCA DO-362A, Command and Control (C2) link robustness is maintained through a four-tier Primary, Alternate, Contingency, Emergency (PACE) communications architecture with automated failover and hysteresis stabilization.

| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Failover Hysteresis | Priority / Role | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary** | Point-to-Point COFDM | 5.8 GHz ISM (5725–5875 MHz) | 15.0 Mbps | 1.5 s | 500 ms | Full Payload HD Video & High-Rate 100 Hz Telemetry | NATO STANAG 4586 §4.5 |
| **Alternate** | Frequency-Hopping Spread Spectrum (FHSS) | 915 MHz ISM (902–928 MHz) | 115.2 kbps | 3.0 s | 1000 ms | Long-Range Robust Flight Telemetry & Tactical C2 | MIL-STD-188-220E §5.3 |
| **Contingency** | Cellular LTE / 5G VPN & SATCOM Relay | Band 28 (700 MHz) & Iridium L-Band (1.6 GHz) | 256.0 kbps / 2.4 kbps | 5.0 s | 2000 ms | Beyond-Line-Of-Sight (BLOS) Telemetry & Safety Commands | RTCA DO-362A §2.2 |
| **Emergency** | Autonomous Return-to-Home & UHF Beacon | 433 MHz UHF / Internal Autonomous Flight Exec | 1.2 kbps / Autonomous | 10.0 s | 5000 ms | Autonomous Lost-Link Rally Navigation & Emergency Beacon | JARUS SORA v2.5 Annex E |

### 5.1 Failover State Transition Dynamics

Link state degradation triggers deterministic down-tier failover when the active tier's heartbeat packet stream is interrupted beyond its designated timeout threshold $\tau_{\mathrm{timeout},i}$:

$$
\begin{aligned}
\Delta t_{\mathrm{loss}}(t) &= t - t_{\mathrm{last\_valid\_rx}} \\
\mathrm{State}(t) &= \begin{cases}
\mathrm{Tier}_i & \text{if } \Delta t_{\mathrm{loss}} < \tau_{\mathrm{timeout},i} \\
\mathrm{Tier}_{i+1} & \text{if } \Delta t_{\mathrm{loss}} \ge \tau_{\mathrm{timeout},i} \quad \text{for } t \ge t_{\mathrm{fail}} + \tau_{\mathrm{hysteresis},i+1}
\end{cases}
\end{aligned}
$$

Where and Operational Parameters:

| Parameter | Symbol | Nominal Value | Units | Constraint / Rule |
| :--- | :--- | :--- | :--- | :--- |
| Active Link Loss Duration | Delta t_loss | Dynamic | s | Measured elapsed duration since last authenticated frame |
| Primary Heartbeat Timeout | tau_timeout,Primary | 1.5 | s | Timeout triggering fallback to Alternate tier |
| Alternate Heartbeat Timeout | tau_timeout,Alternate | 3.0 | s | Timeout triggering fallback to Contingency tier |
| Contingency Heartbeat Timeout | tau_timeout,Contingency | 5.0 | s | Timeout triggering fallback to Emergency tier |
| Emergency Heartbeat Timeout | tau_timeout,Emergency | 10.0 | s | Timeout initiating definitive autonomous return-to-base |
| Re-acquisition Hysteresis Window | tau_hysteresis | 5.0 | s | Continuous stable link duration required before up-tier promotion |

### 5.2 Cryptographic Security & Link Integrity
All PACE communication tiers employ hardware-accelerated AES-256-GCM authenticated encryption with monotonic sequence counters to eliminate eavesdropping, man-in-the-middle packet tampering, and replay injection in compliance with NIST SP 800-82r3.
