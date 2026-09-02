| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Mission Essential Task List |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 2. Mission Essential Task List (METL)

The Mission Essential Task List (METL) establishes the doctrinally grounded, quantitative task inventory required to achieve the Commander's operational intent across all operational lifecycle phases in accordance with CJCSI 3500.02H and NATO STANAG 4586 Annex B.

| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `MET-01` | PreFlightSystemCheckout | Pre-launch stationary ground state with active battery and avionics power | 100% PBIT pass, IMU bias convergence < 30.0 s, battery SoC >= 95.0% | Automated BIT Log Inspection | `/// OperationalAllocation: [MET-01]` | NATO STANAG 4586 Annex B §4.1 |
| `MET-02` | AutonomousIngressTransit | En-route corridor navigation under active flight plan | Lateral cross-track error <= 2.0 m, vertical error <= 1.5 m, airspeed hold +/- 0.5 m/s | RTK GNSS & Flight Log Analysis | `/// OperationalAllocation: [MET-02]` | RTCA DO-365B §2.2.3 |
| `MET-03` | AreaSurveillanceOrbit | On-station target tracking within designated geographic boundary | Orbit radius 100.0 m +/- 5.0 m, payload pointing jitter <= 0.05 deg | Telemetry & Video Frame Analysis | `/// OperationalAllocation: [MET-03]` | NATO STANAG 4586 Annex B §3.2 |
| `MET-04` | DynamicAirspaceDeconfliction | Traffic conflict geometry detected by onboard DAA sensors | Maintain Well-Clear separation (horizontal >= 1200 m, vertical >= 137 m) | DAA Collision Event Log Review | `/// OperationalAllocation: [MET-04]` | RTCA DO-365B §2.2.4 |
| `MET-05` | ElectronicCounterCountermeasures | Jamming or intentional RF / GNSS spoofing detected | Seamless PACE tier switch <= 500 ms, dead-reckoning drift <= 0.5% distance | RF Signal Log & Navigation Test | `/// OperationalAllocation: [MET-05]` | MIL-STD-461G RS103 §4.3 |
| `MET-06` | PrecisionTargetMensuration | Target identified within optical/infrared payload field of view | Target Location Error (TLE) CEP90 <= 3.0 m at 1.5 km slant range | Optical Mensuration Calibration Log | `/// OperationalAllocation: [MET-06]` | NATO STANAG 4586 Annex B §3.5 |
| `MET-07` | AutonomousBingoReturnDivert | Energy monitoring detects current energy <= E_bingo threshold | Autonomous RTB or divert trigger <= 200 ms, touchdown with E_reserve >= 20.0% | BMS Telemetry & Flight Recording | `/// OperationalAllocation: [MET-07]` | JARUS SORA v2.5 Annex E §2.1 |
| `MET-08` | PostMissionDataOffloadShutdown | Post-landing stationary touchdown on recovery pad | Cryptographic key zeroization and data offload in <= 60.0 s, safe shutdown | Security Audit Log Verification | `/// OperationalAllocation: [MET-08]` | NIST SP 800-88r1 §2.4 |

### 2.1 METL Execution Invariants
1. **Deterministic State Progression:** Progression from `MET-01` through `MET-08` is governed by discrete finite-state transitions in the flight executive.
2. **Allocation Tag Integrity:** Every task defined in the METL table carries an inviolable Gate 24 allocation tag (`/// OperationalAllocation: [MET-XX]`) mapped downstream to SysML v2 architectural activity blocks.
3. **Safety Interlock Coupling:** Non-compliance with standard metrics in tasks `MET-01` through `MET-07` initiates deterministic degraded mode failsafes in accordance with Section 8 (Go/No-Go Decision Matrix).
