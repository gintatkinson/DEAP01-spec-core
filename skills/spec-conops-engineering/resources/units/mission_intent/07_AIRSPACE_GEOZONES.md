| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Airspace Deconfliction & Geo-Zones |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones

In accordance with RTCA DO-365B, JARUS SORA v2.5 (Annex C), and EU Regulation 2021/664 (U-space framework), airspace deconfliction is assured through multi-layered 3D spatial containment boundaries, dynamic keep-out zones, and formal Detect and Avoid (DAA) separation minima.

### 7.1 Primary Boundary Perimeter
- **3D Spatial Geometry:** The primary operational airspace is bounded by an authorized convex polygon defined by geodetic coordinates:
  - Vertex 1: $45^\circ 15' 00''\text{ N}, 014^\circ 20' 00''\text{ E}$
  - Vertex 2: $45^\circ 15' 00''\text{ N}, 014^\circ 35' 00''\text{ E}$
  - Vertex 3: $45^\circ 05' 00''\text{ N}, 014^\circ 35' 00''\text{ E}$
  - Vertex 4: $45^\circ 05' 00''\text{ N}, 014^\circ 20' 00''\text{ E}$
- **Vertical Altitude Envelope:** $30.0\text{ m}$ AGL floor up to $120.0\text{ m}$ AGL ceiling ($300.0\text{ m}$ MSL maximum).
- **Containment Buffers:**
  - Lateral Containment Buffer: $50.0\text{ m}$ inward margin along all perimeter boundaries.
  - Vertical Containment Buffer: $15.0\text{ m}$ downward margin below operational ceiling.
  - Trigger Action: Reaching the containment buffer initiates immediate autonomous closed-loop turnaround heading commands.
- **Public Clause Citation:** JARUS SORA v2.5 Annex B §2.1

---

### 7.2 Dynamic Exclusion Zones (Pop-Up Geo-Zones)
- **Dynamic U-space Geofences:** U-space tactical updates broadcast via ADS-B In or cellular C2 telemetry inject dynamic cylindrical or polygonal keep-out volumes into the flight guidance computer.
- **Critical Infrastructure Standoff:** Automated $300.0\text{ m}$ radius cylindrical exclusion zone ($0\text{ to }500\text{ m}$ AGL) around critical energy grids, hospitals, and populated assemblies.
- **Temporary Flight Restrictions (TFR):** Real-time ingestion of NOTAM/TFR spatial polygons with automated dynamic trajectory re-planning.
- **Public Clause Citation:** EU Regulation 2021/664 Art 4

---

### 7.3 RTCA DO-365B DAA Separation Minima (Well-Clear Envelope)
Deconfliction from cooperative (ADS-B / Mode S) and non-cooperative (primary radar / optical) traffic maintains the RTCA DO-365B Well-Clear boundary at all times:

| DAA Separation Parameter | DO-365B Symbol | Nominal Threshold | Units | Operational Definition |
| :--- | :--- | :--- | :--- | :--- |
| Horizontal Miss Distance | DMOD | 1200.0 | m | Minimum horizontal distance allowed at closest point of approach |
| Modified Tau (Time-to-CPA) | tau_mod | 35.0 | s | Temporal collision hazard threshold triggering evasive maneuvers |
| Vertical Separation Minima | HSEP | 137.0 (450 ft) | m | Minimum vertical boundary separation between aircraft flight levels |
| Collision Alert Warning Time | t_alert | 30.0 | s | Advance warning threshold for automated avoidance trajectory execution |

- **Avoidance Maneuver Rule:** If an intruder breaches the DAA Well-Clear envelope, the flight management system executes a coordinated horizontal turn ($3.0^\circ/\text{s}$ turn rate) or vertical climb/dive maneuver prioritizing separation gain.
- **Public Clause Citation:** RTCA DO-365B §2.2.4

---

### 7.4 Terrain Avoidance Envelopes (CFIT Prevention)
- **Digital Elevation Model (DEM) Resolution:** 1-arc-second DTED-2 digital elevation matrix loaded into non-volatile avionics memory.
- **Forward Look-Ahead Horizon:** Forward-looking look-ahead time ($t_{\mathrm{lookahead}} \ge 15.0\text{ s}$) at current ground speed ($v_{\mathrm{ground}}$).
- **Minimum Obstacle Clearance:** Continuous ground and obstacle clearance of $h_{\mathrm{obs\_clear}} \ge 50.0\text{ m}$ along the forward velocity vector.
- **Terrain Avoidance Failsafe:** Detection of terrain penetration along projected 15-second trajectory commands immediate max-power climb at best climb angle ($\gamma_{\mathrm{climb}} = 12.0^\circ$).
- **Public Clause Citation:** ICAO Annex 2 §3.2
