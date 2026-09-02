| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Go/No-Go Decision Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 8. Go/No-Go Decision Matrix

In accordance with NATO STANAG 4586 Annex B (§4.1), RTCA DO-365B, and MIL-STD-882E (§4.3), operational transition across mission phases requires strict logical conjunction satisfaction across pre-flight and in-flight operational safety gates.

| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `GNG-01` | Pre-Launch | Energy Module State of Charge (SoC) | SoC >= SoC_launch_min and Cell Voltage Disparity delta_V_cell <= delta_V_cell_max | Smart BMS Telemetry | Abort Launch if SoC < SoC_launch_min or Disparity > delta_V_cell_max | ASTM F3298-19 §5.4 |
| `GNG-02` | Pre-Launch | Satellite Positioning Geometry | PDOP <= PDOP_max and Visible Satellites N_sat >= N_sat_min and RAIM Valid | Multi-Band Satellite Navigation Receiver | Hold Launch until satellite geometry converges | RTCA DO-365B §2.3 |
| `GNG-03` | Pre-Launch | Environmental Winds & Gusts | v_wind <= v_wind_launch_limit and v_gust <= v_gust_launch_limit | Ground Weather Station Anemometer | Hold Launch if wind velocity exceeds limits | JARUS SORA v2.5 §2.1 |
| `GNG-04` | Pre-Launch | Operational Density Altitude | h_density <= h_density_max | Barometric Pressure & Temperature Sensor | Abort Launch if density altitude > h_density_max | NATO STANAG 4586 Annex B §4.1 |
| `GNG-05` | Pre-Launch | Primary C2 RF Link Margin | Link Margin >= Margin_RF_min and RSSI >= RSSI_min | Datalink Transceiver Diagnostics | Abort Launch if RF link margin < Margin_RF_min | NATO STANAG 4586 §4.5 |
| `GNG-06` | In-Flight | Navigation Figure of Merit (FOM) | FOM <= FOM_max and Horizontal Protection Level HPL <= HPL_max | Extended Kalman Filter State Estimator | Revert to VIO / Initiate Loiter if FOM > FOM_max | RTCA DO-365B §2.2.3 |
| `GNG-07` | In-Flight | Propulsion & Inverter Thermal Envelope | T_ESC <= T_ESC_max and T_Motor <= T_Motor_max | Digital Thermistor Bus | Throttle back / Divert to Secondary Base if T > limit | MIL-STD-882E §4.3 |
| `GNG-08` | In-Flight | Dynamic Geofence Containment Margin | Distance to Boundary d_boundary >= d_containment_margin | Geospatial Containment Filter | Execute immediate 180 deg turn if d_boundary < d_containment_margin | JARUS SORA v2.5 Annex B |

### 8.1 Conjunction Logic and Override Policy
- **Pre-Launch Conjunction:** $\mathrm{Launch\_Go} \iff \bigwedge_{i=1}^{5} \mathrm{GNG}_{i} = \mathrm{TRUE}$. A failure of any single pre-launch gate automatically places the system into a hardware-locked `Hold` state.
- **In-Flight Safety Gate Response:** $\mathrm{Flight\_Continue} \iff \bigwedge_{i=6}^{8} \mathrm{GNG}_{i} = \mathrm{TRUE}$. Any in-flight gate breach triggers immediate autonomous containment in accordance with Section 9 and Section 10.
- **Safety Officer Authority:** Gate overrides are strictly prohibited for safety-critical interlocks (`GNG-01`, `GNG-05`, `GNG-08`) under MIL-STD-882E.
