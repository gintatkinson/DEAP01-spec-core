<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Hierarchical Concept of Operations & Tactical Mission Intent Integrity

**ALWAYS enforce:** All Concept of Operations (`docs/conops/units/conops/` and `CONOPS.md`) and Tactical Mission Intent (`docs/conops/units/mission_intent/` and `MISSION_INTENT.md`) specifications in the Digital Engineering Autonomous Pipeline (DEAP) MUST adhere strictly to pure open schema contracts ($N \ge N_{\mathrm{min}}$ with zero static row caps), open multi-domain threat taxonomies across all 7 operational domains, mandatory INCOSE SEH v5.0 MoE/MoP mathematical formulations, 100% public clause citations, and deterministic modular assembly via `scripts/assemble_conops.py`.

## Scope and Normative Authority

**This file is the single normative home for Concept of Operations (ConOps) and Tactical Mission Intent integrity rules across the DEAP framework.**

ConOps and Mission Intent specifications (Level 1B) operate as the authoritative digital bridge between high-level operational intent and downstream structural extraction (Level 2 Epics, Features, User Stories, and Use Cases) and Model-Based Design (MBD) synthesis.

This governance standard is aligned with:
- **ISO/IEC/IEEE 15288:2023**: Systems and software engineering -- System life cycle processes (§6.4.2 Stakeholder Needs and Requirements Definition & §6.4.3 Architecture Definition Process).
- **INCOSE Systems Engineering Handbook v5.0**: Concept Definition (§3.4.4), Measures of Effectiveness (MoE), Measures of Performance (MoP), and Operational Scenario Engineering.
- **ISO/IEC/IEEE 29148:2018**: Systems and software engineering -- Requirements engineering (§6.4.2 ConOps and §6.4.3 OpsCon).
- **NATO STANAG 4586**: Standard Interfaces of UAV Control System (UCS) for NATO UAV Interoperability.
- **MIL-STD-882E**: Department of Defense Standard Practice: System Safety and Hazard Analysis (Task 202: Operational Hazard Analysis).
- **SAE ARP4761 / ARP4754A**: Guidelines and Methods for Conducting the Safety Assessment Process on Civil Airborne Systems and Equipment (§3: Functional Hazard Assessment).
- **MIL-STD-1629A**: Procedures for Performing a Failure Mode, Effects and Criticality Analysis (Method 101: Functional FMECA, Method 102: Piece-Part Hardware FMECA).
- **JARUS SORA v2.5**: Specific Operations Risk Assessment, 4D Operational Volume, and Ground Risk Buffer (GRB) calculation.
- **OMG UAF v1.2 / v2.0**: Unified Architecture Framework Operational Domain Views (Op-Pr, Op-Tx, Op-Is).

Enforced offline by:
- `parity_auditor/validators/conops_completeness_validator.py` (Gate 26)
- `parity_auditor/validators/coverage_digest_validator.py` (Gate 28)
- `parity_auditor/validators/obligation_witness_validator.py` (Gate 29)

### Architecture Hierarchy & Standards Governance (INCOSE SEH v5.0 §3.4.4 / ISO/IEC/IEEE 15288:2023 / ISO/IEC/IEEE 29148:2018 §6.4.2)

In accordance with **INCOSE Systems Engineering Handbook v5.0 (§3.4.4 Concept Definition)**, **ISO/IEC/IEEE 15288:2023 (§6.4.2 Stakeholder Needs and Requirements Definition & §6.4.3 Architecture Definition Process)**, and **ISO/IEC/IEEE 29148:2018 (§6.4.2 ConOps & §6.4.3 OpsCon)**, the DEAP specification compilation framework enforces a strict 5-tier architecture hierarchy:
1. **Level 1A: Stakeholder Intent, Normative Baseline & Threat Context**: Normative Research Inventory (`docs/research/RESEARCH_INVENTORY.md`), Failure Mode Registry (`docs/research/FAILURE_MODE_REGISTRY.md`), and regulatory obligations baseline.
2. **Level 1B: Concept of Operations (ConOps) & Tactical Mission Intent**: Operational Activities (`OA-01`..`OA-N`), multi-threaded operational scenarios (`SCN-01`..`SCN-N`), operational information exchanges (`OpTx-01`..`OpTx-N`), METL tasks (`MET-01`..`MET-N`), operational lifecycle modes ($\Phi_{\mathrm{lifecycle}}$), SORA 4D containment math, and High-Level Operational Architecture (OV-1 / OV-2 / SV-1).
3. **Level 1C: Logical & Physical Interface Control Documents (ICD)**: Interface Control Documents (`docs/icd/ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `docs/icd/ICD_02_MASTER_SIGNAL_DICTIONARY.md`), discrete port dictionaries, pinouts, bus topologies, and wire framing.
4. **Level 2: System Requirements Specification (SyRS) & Functional Architecture**: Downstream structural requirements consisting of Epics (`epic-xx`), Features (`feat-xx`), User Stories (`us-xx`), and formal System Use Cases (`uc-xx`).
5. **Level 3: Detailed Design & Model-Based Design (MBD)**: Low-level software and hardware requirements, MATLAB / Simulink / Stateflow / Embedded Coder synthesis models, target source code, and Piece-Part BOM FMECA (MIL-STD-1629A Method 102).

### Strict Level 1B vs. Level 2 Metamodel Abstraction Boundary
- **ConOps Operational Scope (Level 1B)**: ConOps models user operational viewpoints, user classes, operational activities (`OA-xx`), and operational scenarios (`SCN-xx`).
- **Strict Exclusion of Level 2 System Use Cases**: ConOps documents and SV-1 diagrams are strictly forbidden from defining, citing, or injecting formal Level 2 System Use Cases (`uc-xx`). Formal System Use Cases (`uc-xx`) specify technical system functions realizing Level 2 Features and belong strictly in downstream System Requirements Specifications (SyRS Level 2 under `docs/use-cases/`).

### System Architecture Diagram Representation Standard (Option 3) & 100% Mathematical Parity Invariant
In accordance with DoDAF v2.02 SV-1, IEEE 1362 §5.3, ISO/IEC/IEEE 15288:2023, INCOSE SE Handbook v5.0 (§3.4.4), and ISO/IEC/IEEE 29148:2018 §6.4.2–§6.4.3:
- **Option 3 Standard (Compact Subsystem Blocks with Embedded Port Attributes)**: Subsystems MUST be rendered as compact subsystem block nodes with embedded bulleted port declarations (`[<b>Name</b><br/>• PORT-... (DIRECTION)]`), rather than exploding ports into separate flowchart nodes within giant subgraphs.
- **Vertical Hierarchical Tier Partitioning (`direction TB`)**: Multi-subsystem and segment interface diagrams MUST declare `direction TB` inside subgraphs and partition nodes into vertical tiers with a maximum of 3 columns horizontally (max 3-column vertical tier partitioning).
- **100% Mathematical Parity with Section 4.8 Allocation Table**: Every subsystem block in the SV-1 diagram embeds the exact set of typed logical and physical ports (`• PORT-... (DIRECTION)`) matching the Section 4.8 Physical & Logical Interface Allocations table rows 1:1, derived deterministically from 100% of declared AST `part def` nodes in exact lockstep.
- **Strict Level 1B vs. Level 1C ICD Boundary**: Detailed internal wire-level interconnects, pinouts, RS-485 serial framing, opcodes (0x10, 0x11, etc.), register bitmasks, and CRC-16 equations belong strictly in Level 1C Interface Control Documents (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`), NOT in the ConOps document. Section 8 (Op-Tx) captures high-level operational information exchanges (Op-Tx: C2 Commands, Telemetry, Video, Target Tracks, Arming Authorization) and strictly excludes component-internal serial opcode reference tables.

---

## The Six Core ConOps & Mission Intent Invariants

### 1. Pure Open Schema Contract ($N \ge N_{\mathrm{min}}$)
- **Zero Static Row Caps**: Artificial upper bounds, hardcoded array limits, or truncation heuristics in tabular specifications or lists are strictly forbidden. All schema contracts defined in `.pipeline/schemas/conops_specification_schema.json` and `.pipeline/schemas/mission_intent_specification_schema.json` are open collections ($N \ge N_{\mathrm{min}}$).
- **Minimum Cardinality Enforcement**: Specifications MUST satisfy domain-specific minimum cardinalities without restriction on upper expansion:
  * **Emergency Decision Matrix**: Minimum 7 canonical triggers ($N \ge 7$) covering `EMG-01` through `EMG-07` (Lost C2, Navigation Loss, Propulsion Failure, Sensor Fault, Geofence Breach, Structural Anomaly, Emergency / Critical Process Termination (Flight Termination / Safe Shutdown)).
  * **PACE C2 Plan**: Minimum 4 communication tiers ($N \ge 4$) covering `Primary`, `Alternate`, `Contingency`, and `Emergency`.
  * **METL Tasks**: Minimum 1 task ($N \ge 1$), expanding to full doctrinal mission scope.
  * **Threat Matrix**: Minimum 1 threat ($N \ge 1$), expanding to all applicable operational threats.
  * **User Classes**: Minimum 1 class ($N \ge 1$).
  * **UAF Operational Activities**: Minimum 1 activity ($N \ge 1$).
  * **Operational Information Exchanges (Op-Tx)**: Minimum 1 exchange ($N \ge 1$).

### 2. Open Multi-Domain Operational Threat Matrices
- **10-Domain Minimum Requirement**: To guarantee comprehensive risk characterization beyond simple mechanical hardware failures, the Multi-Domain Operational Threat & Contested Environment Matrix MUST explicitly address at least one threat vector from every one of the following 10 operational threat domains ($N \ge 10$):
1. **Kinetic**: Ballistic impact, hostile shrapnel, mid-air object collision (e.g., bird strike, debris), explosive blast overpressure.
2. **Mechanical**: Structural flutter, fatigue failure, control surface/actuator jamming, motor bearing seizure, propeller delamination.
3. **Power/Thermal**: Primary battery/fuel cell thermal runaway, BMS fault, catastrophic cell voltage collapse, primary/secondary bus isolation failure.
4. **Environmental**: Extreme ambient temperature, severe turbulence/wind gusts exceeding structural / physical envelope limits, icing/pitot probe freeze, lightning discharge, heavy precipitation, volcanic particulate.
5. **Electronic Warfare (EW)**: Wide-band RF jamming, GNSS spoofing, narrow-band interference, intentional C2 denial, telemetry spoofing.
6. **Cyber**: Denial of Service (DoS), unauthorized cryptographic key rotation, payload buffer overflow exploit, man-in-the-middle (MitM) command insertion, compromised zero-trust endpoints.
7. **Optical**: Camera sensor dazzle/blinding (e.g., directed laser), environmental saturation (sun glare), target occlusion/fog.
8. **Signature**: Unintended radar cross-section (RCS) spike, abnormal acoustic signature emission, unanticipated thermal blooming.
9. **Human Factors**: Ground operator input disparity, operator / pilot fatigue, unauthorized control override, communication protocol desynchronization.
10. **CBRN**: Chemical plumes, biological particulates, radiological contamination, toxic corrosive environments.

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
- **Modular Unit Storage**: ConOps and Mission Intent specifications MUST be authored as discrete modular unit files under `docs/conops/units/conops/` (12 modules: `01_METADATA_AND_OVERVIEW.md` through `12_EMERGENCY_DECISION_MATRIX.md`) and `docs/conops/units/mission_intent/` (10 modules: `01_COMMANDERS_INTENT.md` through `10_OPERATIONAL_ALLOCATION_TAGS.md`).
- **Deterministic Assembly Engine**: Master documents (`docs/conops/CONOPS.md` and `docs/conops/MISSION_INTENT.md`) MUST be compiled via `python3 scripts/assemble_conops.py`.
- **Zero Placeholder Tokens**: Assembled specifications MUST contain zero unresolved `{{...}}` template tokens.
- **Gate 24 Operational Allocation**: Every UAF Operational Activity (`OA-XX`) and METL Task (`MET-XX`) MUST define a machine-verifiable Gate 24 allocation tag (`/// OperationalAllocation: [OA-XX]` or `/// OperationalAllocation: [MET-XX]`) linking operational tasks to structural and behavioral SysML v2 AST elements.

---

## The 3-Tier Multi-Run Safety & Threat Derivation Lifecycle

Safety, threat, and hazard derivation in DEAP follows a strict three-tier, multi-run architectural lifecycle to eliminate circular dependency deadlocks between operational concept definition and physical component selection:

### Tier 1 (Run 1: Mission Intent / Concept Formulation)
- **Primary Methodologies**: Functional Hazard Assessment (FHA) under **SAE ARP4761 §3** and Operational Hazard Analysis (OHA) under **MIL-STD-882E Task 202**.
- **Derivation Mechanism**: Threats and operational hazards are derived systematically from declared **Mission Functions** (`Propel`, `Navigate`, `Communicate`, `Sense`, `Contain`) and **Operating Domains** (`Kinetic`, `Mechanical`, `Power/Thermal`, `Environmental`, `EW`, `Cyber`, `Optical`, `Signature`, `Human Factors`, `CBRN`) using standardized hazard guide words:
  * `Loss`: Complete absence or cessation of the required mission function.
  * `Degraded`: Sub-nominal capacity, reduced bandwidth, or inadequate control authority.
  * `Intermittent`: Sporadic, discontinuous, or jittery functional execution.
  * `Uncommanded`: Inadvertent, unrequested, or anomalous functional activation.
- **Resolution & Scope Rule**: Run 1 **does NOT require piece-part BOM FMECA** (which creates a fatal circular dependency deadlock before physical hardware components are architected and selected). Instead, Run 1 strictly enforces 100% complete FHA and OHA functional hazard coverage across all declared mission functions and operational threat domains.

### Tier 2 (Run 2: ConOps / Logical Architecture & SysML)
- **Primary Methodologies**: Functional Failure Mode, Effects, and Criticality Analysis (FMECA) under **MIL-STD-1629A Method 101** and System-Theoretic Process Analysis (**STPA**).
- **Derivation Mechanism**: Failure modes and safety constraints are derived from logical subsystem blocks, data bus interfaces (e.g. CAN, Ethernet, Serial), inter-subsystem control loops, and operational activity transitions (`OA-*`).
- **Focus**: Logical interface boundaries, feedback loop delays, unsafe control actions (UCAs), and loss of functional redundancy.

### Tier 3 (Run 3: Detailed Engineering & Physical BOM)
- **Primary Methodologies**: Piece-Part Hardware Failure Mode, Effects, and Criticality Analysis (FMECA) under **MIL-STD-1629A Method 102**.
- **Derivation Mechanism**: Calculates quantitative failure rates ($\lambda$) and criticality metrics ($C_r$) for specific physical hardware components, commercial-off-the-shelf (COTS) parts, electrical interconnects, and circuit components based on empirical reliability handbooks (e.g. MIL-HDBK-217F, NPRD-2016).

---

## Standard Metric & Volume Equations

### SORA Ground Risk Buffer Formulation
$$
\begin{aligned}
V_{\mathrm{4D}} &= V_{\mathrm{SpatialGeometry}} \cup V_{\mathrm{ContingencyVolume}} \cup V_{\mathrm{GRB}} \\
R_{\mathrm{GRB}} &= h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}}) + v_{\mathrm{wind,max}} \cdot \sqrt{\frac{2 h_{\mathrm{max}}}{g}} + d_{\mathrm{glide,max}}
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
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

- Parameter Definitions & Engineering Units:
- $E_{\mathrm{bingo}}(t)$: Dynamic Bingo energy threshold triggering immediate return or divert.
- $E_{\mathrm{return}}$: Energy required to transit from current coordinate $\mathbf{p}(t)$ to primary recovery point $\mathbf{p}_{\mathrm{dest}}$.
- $E_{\mathrm{divert}}$: Energy required to divert from primary recovery point to alternate recovery point $\mathbf{p}_{\mathrm{alt}}$.
- $E_{\mathrm{reserve}}$: Statutory reserve energy (must be $\ge 20\%$ of total storage capacity $E_{\mathrm{capacity}}$).
- $E_{\mathrm{contingency}}$: Dynamic holding pattern and wind compensation buffer.

---

## Why

Treating operational concepts and mission intent as informal, ad-hoc prose leads to unverified airspace risks, untraced mission requirements, under-dimensioned safety containment buffers, and catastrophic system failures during contingency events.

Enforcing strict schema contracts, open multi-domain threat modeling, mathematical formulations, public clause citations, and deterministic assembly ensures that Level 1B operational concepts are as rigorous and machine-verifiable as low-level source code.
