| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Airspace Deconfliction & Geo-Zones |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones

In accordance with RTCA DO-365B, JARUS SORA v2.5 (Annex C), and EU Regulation 2021/664 (U-space framework), airspace deconfliction is assured through multi-layered 3D spatial containment boundaries, dynamic keep-out zones, and formal Detect and Avoid (DAA) separation minima.

### 7.1 Primary Boundary Perimeter
- **3D Spatial Geometry:** The primary operational airspace is bounded by an authorized convex polygon defined by parameterized geodetic vertices $\{\mathbf{p}_{\mathrm{vertex},1}, \dots, \mathbf{p}_{\mathrm{vertex},N}\}$.
- **Vertical Altitude Envelope:** Altitude floor $h_{\mathrm{floor}}$ up to operating ceiling $h_{\mathrm{ceiling}}$ (bounded by $h_{\mathrm{MSL\_max}}$).
- **Containment Buffers:**
  - Lateral Containment Buffer: $d_{\mathrm{lateral\_buffer}}$ inward margin along all perimeter boundaries.
  - Vertical Containment Buffer: $d_{\mathrm{vertical\_buffer}}$ downward margin below operational ceiling.
  - Trigger Action: Reaching the containment buffer initiates immediate autonomous closed-loop turnaround heading commands.
- **Public Clause Citation:** JARUS SORA v2.5 Annex B §2.1

---

### 7.2 Dynamic Exclusion Zones (Pop-Up Geo-Zones)
- **Dynamic U-space Geofences:** U-space tactical updates broadcast via datalink inject dynamic cylindrical or polygonal keep-out volumes into the flight guidance computer.
- **Critical Infrastructure Standoff:** Automated exclusion cylinder of radius $R_{\mathrm{exclusion}}$ spanning $[h_{\mathrm{ex\_floor}}, h_{\mathrm{ex\_ceiling}}]$ around critical infrastructure and populated assemblies.
- **Temporary Flight Restrictions (TFR):** Real-time ingestion of NOTAM/TFR spatial polygons with automated dynamic trajectory re-planning.
- **Public Clause Citation:** EU Regulation 2021/664 Art 4

---

### 7.3 RTCA DO-365B DAA Separation Minima (Well-Clear Envelope)
Deconfliction from cooperative and non-cooperative traffic maintains the RTCA DO-365B Well-Clear boundary at all times:

| DAA Separation Parameter | DO-365B Symbol | Nominal Constraint | Units | Operational Definition |
| :--- | :--- | :--- | :--- | :--- |
| Horizontal Miss Distance | DMOD | DMOD >= DMOD_min | m | Minimum horizontal distance allowed at closest point of approach |
| Modified Tau (Time-to-CPA) | tau_mod | tau_mod >= tau_mod_min | s | Temporal collision hazard threshold triggering evasive maneuvers |
| Vertical Separation Minima | HSEP | HSEP >= HSEP_min | m | Minimum vertical boundary separation between aircraft flight levels |
| Collision Alert Warning Time | t_alert | t_alert >= tau_alert_min | s | Advance warning threshold for automated avoidance trajectory execution |

- **Avoidance Maneuver Rule:** If an intruder breaches the DAA Well-Clear envelope, the flight management system executes a coordinated horizontal turn at turn rate $\omega_{\mathrm{turn\_rate}}$ or vertical climb/dive maneuver prioritizing separation gain.
- **Public Clause Citation:** RTCA DO-365B §2.2.4

---

### 7.4 Terrain Avoidance Envelopes (CFIT Prevention)
- **Digital Elevation Model (DEM) Resolution:** High-resolution digital elevation matrix loaded into non-volatile avionics memory.
- **Forward Look-Ahead Horizon:** Forward look-ahead time ($t_{\mathrm{lookahead}} \ge \tau_{\mathrm{lookahead\_min}}$) at current ground speed ($v_{\mathrm{ground}}$).
- **Minimum Obstacle Clearance:** Continuous obstacle clearance of $h_{\mathrm{obs\_clear}} \ge h_{\mathrm{clearance\_min}}$ along the forward velocity vector.
- **Terrain Avoidance Failsafe:** Detection of terrain penetration along projected trajectory commands immediate maximum-power climb at best climb angle ($\gamma_{\mathrm{climb}} \ge \gamma_{\mathrm{climb\_min}}$).
- **Public Clause Citation:** ICAO Annex 2 §3.2
