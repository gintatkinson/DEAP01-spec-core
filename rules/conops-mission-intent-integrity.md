<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Hierarchical Concept of Operations & Tactical Mission Intent Integrity

**ALWAYS enforce:** All Concept of Operations (`docs/conops/units/conops/` and `CONOPS.md`) and Tactical Mission Intent (`docs/conops/units/mission_intent/` and `MISSION_INTENT.md`) specifications in the Digital Engineering Autonomous Pipeline (DEAP) MUST adhere strictly to pure open schema contracts ($N \ge N_{\mathrm{min}}$ with zero static row caps), open multi-domain threat taxonomies across all 7 operational domains, mandatory INCOSE SEH v5.0 MoE/MoP mathematical formulations, 100% public clause citations, and deterministic modular assembly via `scripts/assemble_conops.py`.

## Scope and Normative Authority

**This file is the single normative home for Concept of Operations (ConOps) and Tactical Mission Intent integrity rules across the DEAP framework.**

ConOps and Mission Intent specifications (Level 1B) operate as the authoritative digital bridge between high-level operational intent and downstream structural extraction (Level 2 Epics, Features, User Stories, and Use Cases) and Model-Based Design (MBD) synthesis.

This governance standard is aligned with:
- **ISO/IEC/IEEE 29148:2018**: Systems and software engineering — Requirements engineering (§6.4.2 ConOps and §6.4.3 OpsCon).
- **INCOSE Systems Engineering Handbook v5.0**: Measures of Effectiveness (MoE), Measures of Performance (MoP), and Operational Scenario Engineering.
- **NATO STANAG 4586**: Standard Interfaces of UAV Control System (UCS) for NATO UAV Interoperability.
- **MIL-STD-882E**: Department of Defense Standard Practice: System Safety and Hazard Analysis.
- **JARUS SORA v2.5**: Specific Operations Risk Assessment, 4D Operational Volume, and Ground Risk Buffer (GRB) calculation.
- **OMG UAF v1.2 / v2.0**: Unified Architecture Framework Operational Domain Views (Op-Pr, Op-Tx, Op-Is).

Enforced offline by:
- `parity_auditor/validators/conops_completeness_validator.py` (Gate 26)
- `parity_auditor/validators/coverage_digest_validator.py` (Gate 28)
- `parity_auditor/validators/obligation_witness_validator.py` (Gate 29)

---

## The Six Core ConOps & Mission Intent Invariants

### 1. Pure Open Schema Contract ($N \ge N_{\mathrm{min}}$)
- **Zero Static Row Caps**: Artificial upper bounds, hardcoded array limits, or truncation heuristics in tabular specifications or lists are strictly forbidden. All schema contracts defined in `.pipeline/schemas/conops_specification_schema.json` and `.pipeline/schemas/mission_intent_specification_schema.json` are open collections ($N \ge N_{\mathrm{min}}$).
- **Minimum Cardinality Enforcement**: Specifications MUST satisfy domain-specific minimum cardinalities without restriction on upper expansion:
  * **Emergency Decision Matrix**: Minimum 7 canonical triggers ($N \ge 7$) covering `EMG-01` through `EMG-07` (Lost C2, Navigation Loss, Propulsion Failure, Sensor Fault, Geofence Breach, Structural Anomaly, Flight Termination).
  * **PACE C2 Plan**: Minimum 4 communication tiers ($N \ge 4$) covering `Primary`, `Alternate`, `Contingency`, and `Emergency`.
  * **METL Tasks**: Minimum 1 task ($N \ge 1$), expanding to full doctrinal mission scope.
  * **Threat Matrix**: Minimum 1 threat ($N \ge 1$), expanding to all applicable operational threats.
  * **User Classes**: Minimum 1 class ($N \ge 1$).
  * **UAF Operational Activities**: Minimum 1 activity ($N \ge 1$).
  * **Operational Information Exchanges (Op-Tx)**: Minimum 1 exchange ($N \ge 1$).

### 2. Open Multi-Domain Threat Taxonomy
The Threat and Electronic Warfare / Cyber Environment Matrix MUST cover multi-domain threats across all seven canonical operational domains:
1. **Kinetic**: External projectiles, mid-air collisions, physical interceptors, ballistic fragmentation, ground obstacles.
2. **Mechanical**: Structural flutter, fatigue failure, control surface/actuator jamming, motor bearing seizure, propeller delamination.
3. **Environmental**: Extreme ambient temperature, severe turbulence/wind gusts exceeding airframe limits, icing/pitot probe freeze, lightning discharge, heavy precipitation, volcanic particulate.
4. **EW / Cyber**: GNSS spoofing/jamming, RF command uplink jamming, telemetry sniffing, man-in-the-middle packet injection, unauthorized command injection, firmware tampering.
5. **Power / Thermal**: Battery cell thermal runaway, power rail brownout, electronic speed controller (ESC) thermal throttling, generator disconnect.
6. **Optical**: High-energy laser blinding of optical tracking sensors, sensor dazzling, camera saturation, optical flow denial.
7. **Human**: Ground operator input disparity, pilot fatigue, unauthorized control override, communication protocol desynchronization.

Restricting threat analysis to an arbitrary single domain or omitting applicable threat vectors is strictly prohibited.

### 3. Mandatory INCOSE SEH v5.0 MoE/MoP Mathematical Formulations
- **Mathematical Formulations**: Every Measure of Effectiveness (MoE) and Measure of Performance (MoP) defined in Section 3 of Mission Intent MUST provide an explicit mathematical equation or formula expressing the metric in KaTeX format.
- **Threshold & Objective Value Pairs**: Every metric MUST declare both a minimum acceptable **Threshold** performance value and an optimal **Objective** target value.
- **SI & Normalized Units**: Every metric MUST define its unit of measurement using normalized SI units, percentages, or explicit `Dimensionless` designations.

### 4. 100% Public Clause Citations
- **Traceability Rule**: Every operational requirement, normative standard, threat mitigation rule, and METL task MUST cite verifiable, publicly accessible standard clauses (e.g., `ISO/IEC/IEEE 29148:2018 §6.4.2`, `NATO STANAG 4586 Annex B §3.2`, `JARUS SORA v2.5 Annex B §2.1`, `MIL-STD-882E §4.3`, `RTCA DO-178C §6.3.1`).
- **Prohibition of Un-Cited Additions**: Speculative, ungrounded, or heuristic requirements lacking authoritative clause citations are strictly forbidden across all specification units.

### 5. LaTeX & KaTeX Mathematical Rendering Integrity
Per [`rules/latex-katex-integrity.md`](latex-katex-integrity.md):
- **Display Math Blocks**: All mathematical formulations (SORA 4D Ground Risk Buffer $R_{\mathrm{GRB}}$, Bingo Energy dynamics $E_{\mathrm{bingo}}(t)$) MUST use dedicated display blocks `$$ \begin{aligned} ... \end{aligned} $$` on separate newlines.
- **Pure Symbolic Math**: Embedding physical unit macros (e.g. `\text{ m}`, `\text{ m/s}`, `\text{ J}`) inside LaTeX math blocks is strictly prohibited.
- **Accompanying Parameter Tables**: Display equations MUST be immediately followed by a parameter definition table defining every mathematical symbol, nominal value, SI unit, and engineering constraint.
- **No Table Math Delimiters**: Math delimiters (`$ ... $` or `$$ ... $$`) inside Markdown table headers, delimiter rows, and data cells are strictly prohibited. Use standard plain text and Unicode characters (e.g. `h_max m`, `v_wind m/s`, `J`, `deg`, `tau_max ms`).

### 6. Deterministic Modular Assembly & Cross-Model Allocation
- **Modular Unit Storage**: ConOps and Mission Intent specifications MUST be authored as discrete modular unit files under `docs/conops/units/conops/` (12 modules: `01_scope.md` through `12_emergency_matrix.md`) and `docs/conops/units/mission_intent/` (10 modules: `01_intent.md` through `10_tags.md`).
- **Deterministic Assembly Engine**: Master documents (`docs/conops/CONOPS.md` and `docs/conops/MISSION_INTENT.md`) MUST be compiled via `python3 scripts/assemble_conops.py`.
- **Zero Placeholder Tokens**: Assembled specifications MUST contain zero unresolved `{{...}}` template tokens.
- **Gate 24 Operational Allocation**: Every UAF Operational Activity (`OA-XX`) and METL Task (`MET-XX`) MUST define a machine-verifiable Gate 24 allocation tag (`/// OperationalAllocation: [OA-XX]` or `/// OperationalAllocation: [MET-XX]`) linking operational tasks to structural and behavioral SysML v2 AST elements.

---

## Standard Metric & Volume Equations

### SORA Ground Risk Buffer Formulation
$$
\begin{aligned}
V_{\mathrm{4D}} &= V_{\mathrm{SpatialGeometry}} \cup V_{\mathrm{ContingencyVolume}} \cup V_{\mathrm{GRB}} \\
R_{\mathrm{GRB}} &= h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}}) + v_{\mathrm{wind,max}} \cdot \sqrt{\frac{2 h_{\mathrm{max}}}{g}} + d_{\mathrm{glide,max}}
\end{aligned}
$$

Where and Operational Parameters:
- $V_{\mathrm{4D}}$: Total 4D spatial-temporal operational volume envelope.
- $R_{\mathrm{GRB}}$: Declared ground risk buffer radius.
- $h_{\mathrm{max}}$: Maximum operating altitude above reference surface (m).
- $\theta_{\mathrm{impact}}$: Worst-case operational trajectory impact angle (deg).
- $v_{\mathrm{wind,max}}$: Maximum operational wind speed limit (m/s).
- $g$: Standard gravitational acceleration constant ($9.80665\text{ m/s}^2$).
- $d_{\mathrm{glide,max}}$: Maximum unpowered lateral displacement margin (m).

### Bingo Energy Dynamics Formulation
$$
\begin{aligned}
E_{\mathrm{bingo}}(t) &= E_{\mathrm{return}}(\mathbf{p}(t), \mathbf{p}_{\mathrm{dest}}) + E_{\mathrm{divert}}(\mathbf{p}_{\mathrm{dest}}, \mathbf{p}_{\mathrm{alt}}) + E_{\mathrm{reserve}} + E_{\mathrm{contingency}} \\
E_{\mathrm{reserve}} &\ge 0.20 \cdot E_{\mathrm{capacity}}
\end{aligned}
$$

Where and Operational Parameters:
- $E_{\mathrm{bingo}}(t)$: Dynamic Bingo energy threshold triggering immediate return or divert.
- $E_{\mathrm{return}}$: Energy required to transit from current coordinate $\mathbf{p}(t)$ to primary recovery point $\mathbf{p}_{\mathrm{dest}}$.
- $E_{\mathrm{divert}}$: Energy required to divert from primary recovery point to alternate recovery point $\mathbf{p}_{\mathrm{alt}}$.
- $E_{\mathrm{reserve}}$: Statutory reserve energy (must be $\ge 20\%$ of total storage capacity $E_{\mathrm{capacity}}$).
- $E_{\mathrm{contingency}}$: Dynamic holding pattern and wind compensation buffer.

---

## Why

Treating operational concepts and mission intent as informal, ad-hoc prose leads to unverified airspace risks, untraced mission requirements, under-dimensioned safety containment buffers, and catastrophic system failures during contingency events.

Enforcing strict schema contracts, open multi-domain threat modeling, mathematical formulations, public clause citations, and deterministic assembly ensures that Level 1B operational concepts are as rigorous and machine-verifiable as low-level source code.
