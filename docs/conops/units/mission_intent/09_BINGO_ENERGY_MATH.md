| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent: Bingo Energy Mathematics |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 9. Bingo Energy Mathematics & Secondary Divert Protocols

In accordance with JARUS SORA v2.5 (Annex E §2.1), NATO STANAG 4586 Annex B (§3.4), and EASA GM1 UAS.SPEC.050(1)(g), safe recovery is guaranteed by continuous parametric computation of the dynamic Bingo Energy state $E_{\mathrm{bingo}}(t)$ and enforcement of the mandatory 20.0% statutory reserve threshold.

### 9.1 Parametric Closed-Loop Bingo Formulation

The dynamic Bingo energy threshold represents the minimum onboard stored energy required to transit from the current operating position to the primary destination, divert to an alternate recovery point if the primary site is fouled, and complete landing with statutory safety reserves intact:

$$
\begin{aligned}
E_{\mathrm{bingo}}(t) &= E_{\mathrm{return}}(\mathbf{p}(t), \mathbf{p}_{\mathrm{dest}}) + E_{\mathrm{divert}}(\mathbf{p}_{\mathrm{dest}}, \mathbf{p}_{\mathrm{alt}}) + E_{\mathrm{reserve}} + E_{\mathrm{contingency}} \\
E_{\mathrm{reserve}} &\ge 0.20 \cdot E_{\mathrm{capacity}}
\end{aligned}
$$

The dynamic transit and divert components are evaluated via continuous integral path energy modeling:

$$
\begin{aligned}
E_{\mathrm{return}}(\mathbf{p}(t), \mathbf{p}_{\mathrm{dest}}) &= \int_{t}^{t_{\mathrm{land}}} \left( P_{\mathrm{prop}}(v_{\mathrm{airspeed}}(\tau), \mathbf{v}_{\mathrm{wind}}(\tau)) + P_{\mathrm{avionics}} + P_{\mathrm{payload}} \right) d\tau \\
E_{\mathrm{divert}}(\mathbf{p}_{\mathrm{dest}}, \mathbf{p}_{\mathrm{alt}}) &= \frac{\|\mathbf{p}_{\mathrm{dest}} - \mathbf{p}_{\mathrm{alt}}\|_2}{v_{\mathrm{cruise}}} \cdot P_{\mathrm{cruise}} + E_{\mathrm{climb\_divert}}
\end{aligned}
$$

Where and Operational Parameters:

| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | 500000.0 | J | Total nominal battery energy storage | JARUS SORA v2.5 Annex E |
| Return Transit Energy | E_return | 150000.0 | J | Energy required for nominal return trajectory | NATO STANAG 4586 Annex B §3.4 |
| Secondary Divert Energy | E_divert | 60000.0 | J | Energy required for divert transit to alternate site | NATO STANAG 4586 Annex B §3.4 |
| Mandatory Statutory Reserve | E_reserve | 100000.0 | J | E_reserve >= 0.20 * E_capacity (20.0% minimum) | EASA GM1 UAS.SPEC.050(1)(g) |
| Contingency Buffer | E_contingency | 40000.0 | J | Dynamic wind compensation & holding pattern buffer | JARUS SORA v2.5 Annex E |
| Total Bingo Threshold | E_bingo | 350000.0 | J | Critical return trigger: E_current <= E_bingo -> RTB | NATO STANAG 4586 Annex B §3.4 |
| Calculated Reserve Ratio | Ratio_reserve | 0.200 | Dimensionless | Ratio_reserve = E_reserve / E_capacity >= 0.200 | EASA GM1 UAS.SPEC.050(1)(g) |

---

### 9.2 Secondary and Tertiary Divert Recovery Protocols

When the primary landing zone is unavailable (runway incursion, localized weather hazard, or C2 loss), the flight manager autonomously executes secondary or tertiary divert sequencing:

| Recovery Site ID | Site Classification | Geodetic Location | Elevation | Runway / Pad Dimension | Priority Order | Ingress Clearance |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `LZ-PRIMARY` | Primary Base Recovery Pad | 45° 10' 15" N, 014° 25' 30" E | 45.0 m MSL | 20 m x 20 m Concrete Pad | Priority 1 (Nominal) | Unrestricted Line-of-Sight |
| `LZ-DIVERT-ALPHA` | Secondary Divert Field | 45° 12' 40" N, 014° 22' 10" E | 62.0 m MSL | 50 m Grass Strip | Priority 2 (Secondary) | 30 m Tree Clearance |
| `LZ-DIVERT-BRAVO` | Tertiary Emergency Clearing | 45° 08' 20" N, 014° 30' 00" E | 28.0 m MSL | 30 m Unpaved Clearing | Priority 3 (Contingency) | High-Tension Wire Buffer 500 m |

- **Autonomous Divert Protocol:** If $E_{\mathrm{current}} \le E_{\mathrm{bingo}}$ and `LZ-PRIMARY` reports obstruction, the flight executive commands immediate divert to `LZ-DIVERT-ALPHA` within $\le 200\text{ ms}$, adjusting climb profile and airspeed for optimal specific air range (SAR).
- **Public Clause Citation:** NATO STANAG 4586 Annex B §3.4
