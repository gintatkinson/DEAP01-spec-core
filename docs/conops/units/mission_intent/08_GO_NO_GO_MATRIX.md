| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Go/No-Go Decision Matrix |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 8. Go/No-Go Decision Matrix

In accordance with NATO STANAG 4586 Annex B (§4.1), RTCA DO-365B, and MIL-STD-882E (§4.3), operational transition across mission phases requires strict logical conjunction satisfaction across pre-flight and in-flight operational safety gates.

| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `GNG-01` | Pre-Launch | Battery State of Charge (SoC) | SoC >= 95.0% and Cell Voltage Disparity <= 20 mV | Smart BMS Telemetry | Abort Launch if SoC < 95.0% or Disparity > 20 mV | ASTM F3298-19 §5.4 |
| `GNG-02` | Pre-Launch | Satellite Constellation Geometry | PDOP < 2.5 and Visible Satellites >= 10 and RAIM Valid | Multi-Band RTK GNSS Receiver | Hold Launch until satellite geometry converges | RTCA DO-365B §2.3 |
| `GNG-03` | Pre-Launch | Environmental Crosswinds & Gusts | v_wind <= 12.0 m/s and v_gust <= 15.0 m/s | Ground Weather Station Anemometer | Hold Launch if wind velocity exceeds limits | JARUS SORA v2.5 §2.1 |
| `GNG-04` | Pre-Launch | Airfield Density Altitude | h_density <= 3000.0 m MSL | Barometric Pressure & Temperature Sensor | Abort Launch if density altitude > 3000.0 m | NATO STANAG 4586 Annex B §4.1 |
| `GNG-05` | Pre-Launch | Primary C2 RF Link Margin | Link Margin >= 12.0 dB and RSSI >= -75 dBm | 5.8 GHz COFDM Transceiver Diagnostics | Abort Launch if RF link margin < 12.0 dB | NATO STANAG 4586 §4.5 |
| `GNG-06` | In-Flight | Navigation Figure of Merit (FOM) | FOM <= 2.0 and Horizontal Protection Level <= 10.0 m | Extended Kalman Filter State Estimator | Revert to VIO / Initiate Loiter if FOM > 2.0 | RTCA DO-365B §2.2.3 |
| `GNG-07` | In-Flight | Propulsion & Inverter Thermal Envelope | T_ESC <= 85.0 degC and T_Motor <= 90.0 degC | Digital Thermistor Bus | Throttle back / Divert to Secondary Base if T > limit | MIL-STD-882E §4.3 |
| `GNG-08` | In-Flight | Dynamic Geofence Containment Margin | Distance to Boundary d_boundary >= 50.0 m | Geospatial Containment Filter | Execute immediate 180 deg turn if d_boundary < 50.0 m | JARUS SORA v2.5 Annex B |

### 8.1 Conjunction Logic and Override Policy
- **Pre-Launch Conjunction:** $\mathrm{Launch\_Go} \iff \bigwedge_{i=1}^{5} \mathrm{GNG}_{i} = \mathrm{TRUE}$. A failure of any single pre-launch gate automatically places the system into a hardware-locked `Hold` state.
- **In-Flight Safety Gate Response:** $\mathrm{Flight\_Continue} \iff \bigwedge_{i=6}^{8} \mathrm{GNG}_{i} = \mathrm{TRUE}$. Any in-flight gate breach triggers immediate autonomous containment in accordance with Section 9 and Section 10.
- **Safety Officer Authority:** Gate overrides are strictly prohibited for safety-critical interlocks (`GNG-01`, `GNG-05`, `GNG-08`) under MIL-STD-882E.
