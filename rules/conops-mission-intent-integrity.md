<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Hierarchical Concept of Operations & Tactical Mission Intent Integrity

**ALWAYS enforce:** All Concept of Operations (`docs/conops/units/conops/` and `CONOPS.md`) and Tactical Mission Intent (`docs/conops/units/mission_intent/` and `MISSION_INTENT.md`) specifications in the Digital Engineering Autonomous Pipeline (DEAP) MUST adhere strictly to pure open schema contracts ($N \ge N_{\mathrm{min}}$ with zero static row caps), open multi-domain threat taxonomies across all 9 operational domains, mandatory INCOSE SEH v5.0 MoE/MoP mathematical formulations, 100% public clause citations, and deterministic modular assembly via `./target/release/assemble-conops`.

## Scope and Normative Authority

**This file is the single normative home for Concept of Operations (ConOps) and Tactical Mission Intent integrity rules across the DEAP framework.**

ConOps and Mission Intent specifications (Level 1B) operate as the authoritative digital bridge between high-level operational intent and downstream structural extraction (Level 2 Epics, Features, User Stories, and Use Cases) and Model-Based Design (MBD) synthesis.

This governance standard is aligned with:
- **ISO/IEC/IEEE 15288:2023**: Systems and software engineering -- System life cycle processes (§6.4.2 Stakeholder Needs and Requirements Definition & §6.4.3 Architecture Definition Process).
- **INCOSE Systems Engineering Handbook v5.0**: Concept Definition (§3.4.4), Measures of Effectiveness (MoE), Measures of Performance (MoP), and Operational Scenario Engineering.
- **ISO/IEC/IEEE 29148:2018**: Systems and software engineering -- Requirements engineering (§6.4.2 ConOps and §6.4.3 OpsCon).
- **IEEE 1362:1998**: Guide for Information Technology -- System Definition -- Concept of Operations (ConOps) Document.
- **MIL-STD-882E**: Department of Defense Standard Practice: System Safety and Hazard Analysis (Task 202: Operational Hazard Analysis).
- **IEC 61508**: Functional safety of electrical/electronic/programmable electronic safety-related systems (§7: Management of functional safety & Hazard Analysis).
- **MIL-STD-1629A**: Procedures for Performing a Failure Mode, Effects and Criticality Analysis (Method 101: Functional FMECA, Method 102: Piece-Part Hardware FMECA).
- **Operational Domain Risk Assessment**: 4D Operational Volume and Boundary Containment Margin calculation.
- **OMG UAF v1.2 / v2.0**: Unified Architecture Framework Operational Domain Views (Op-Pr, Op-Tx, Op-Is).

Enforced offline by:
- `./target/release/verify-baseline` (superseding the retired Python validator `conops_completeness_validator.py`) (Gate 26)
- `./target/release/verify-baseline` (superseding the retired Python validator `coverage_digest_validator.py`) (Gate 28)
- `./target/release/verify-baseline` (superseding the retired Python validator `obligation_witness_validator.py`) (Gate 29)

### Architecture Hierarchy & Standards Governance (INCOSE SEH v5.0 §3.4.4 / ISO/IEC/IEEE 15288:2023 / ISO/IEC/IEEE 29148:2018 §6.4.2)

In accordance with **INCOSE Systems Engineering Handbook v5.0 (§3.4.4 Concept Definition)**, **ISO/IEC/IEEE 15288:2023 (§6.4.2 Stakeholder Needs and Requirements Definition & §6.4.3 Architecture Definition Process)**, and **ISO/IEC/IEEE 29148:2018 (§6.4.2 ConOps & §6.4.3 OpsCon)**, the DEAP specification compilation framework enforces a strict 5-tier architecture hierarchy:
1. **Level 1A: Stakeholder Intent, Normative Baseline & Threat Context**: Normative Research Inventory (`docs/research/RESEARCH_INVENTORY.md`), Failure Mode Registry (`docs/research/FAILURE_MODE_REGISTRY.md`), and regulatory obligations baseline.
2. **Level 1B: Concept of Operations (ConOps) & Tactical Mission Intent**: Operational Activities (`OA-01`..`OA-N`), multi-threaded operational scenarios (`SCN-01`..`SCN-N`), operational information exchanges (`OpTx-01`..`OpTx-N`), METL tasks (`MET-01`..`MET-N`), operational lifecycle modes ($\Phi_{\mathrm{lifecycle}}$), 4D operational containment mathematics, and High-Level Operational Architecture (OV-1 / OV-2 / SV-1).
3. **Level 1C: Logical & Physical Interface Control Documents (ICD)**: Interface Control Documents (`docs/icd/ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `docs/icd/ICD_02_MASTER_SIGNAL_DICTIONARY.md`), discrete port dictionaries, pinouts, bus topologies, and wire framing.
4. **Level 2: System Requirements Specification (SyRS) & Functional Architecture**: Downstream structural requirements consisting of Epics (`epic-xx`), Features (`feat-xx`), User Stories (`us-xx`), and formal System Use Cases (`uc-xx`).
5. **Level 3: Detailed Design & Model-Based Design (MBD)**: Low-level software and hardware requirements, MATLAB / Simulink / Stateflow / Embedded Coder synthesis models, target source code, and Piece-Part BOM FMECA (MIL-STD-1629A Method 102).

### Strict Level 1B vs. Level 2 Metamodel Abstraction Boundary
- **ConOps Operational Scope (Level 1B)**: ConOps models user operational viewpoints, user classes, operational activities (`OA-xx`), and operational scenarios (`SCN-xx`).
- **Strict Exclusion of Level 2 System Use Cases**: ConOps documents and SV-1 diagrams are strictly forbidden from defining, citing, or injecting formal Level 2 System Use Cases (`uc-xx`). Formal System Use Cases (`uc-xx`) specify technical system functions realizing Level 2 Features and belong strictly in downstream System Requirements Specifications (SyRS Level 2 under `docs/use-cases/`).
- **User Class Taxonomy Disambiguation (`UCL-xx`)**: In accordance with ISO/IEC/IEEE 29148:2018 §5.2.4, operational User Classes MUST use the `UCL-xx` prefix (`UCL-01` through `UCL-N`), strictly isolating stakeholder taxonomy from SyRS Level 2 System Use Cases (`uc-xx` or `USECASE-xx`) to prevent downstream agent reasoning collisions and automated sanitization corruption (refs #291).


### System Architecture Diagram Representation Standard (Option 3) & 100% Mathematical Parity Invariant
In accordance with DoDAF v2.02 SV-1, IEEE 1362 §5.3, ISO/IEC/IEEE 15288:2023, INCOSE SE Handbook v5.0 (§3.4.4), and ISO/IEC/IEEE 29148:2018 §6.4.2–§6.4.3:
- **Option 3 Standard (Compact Subsystem Blocks with Embedded Port Attributes)**: Subsystems MUST be rendered as compact subsystem block nodes with embedded bulleted port declarations (`[<b>Name</b><br/>• PORT-... (DIRECTION)]`), rather than exploding ports into separate flowchart nodes within giant subgraphs.
- **Vertical Hierarchical Tier Partitioning (`direction TB`)**: Multi-subsystem and segment interface diagrams MUST declare `direction TB` inside subgraphs and partition nodes into vertical tiers with a maximum of 3 columns horizontally (max 3-column vertical tier partitioning).
- **100% Mathematical Parity with Section 4.8 Allocation Table**: Every subsystem block in the SV-1 diagram embeds the exact set of typed logical and physical ports (`• PORT-... (DIRECTION)`) matching the Section 4.8 Physical & Logical Interface Allocations table rows 1:1, derived deterministically from 100% of declared AST `part def` nodes in exact lockstep.
- **Pure Schema-Driven Operational Segments & Dynamic Layout Spacers**: Operational segments and Mermaid layout subgraphs MUST derive 100% and exclusively from top-level packages or segment blocks present in the user-provided SysML AST in `schema/`. Hardcoding domain-specific segment names or communication plans into architecture diagrams, rules, or templates is strictly prohibited. Layout vertical spacers (`~~~`) MUST be generated dynamically between declared AST segments.
- **Level 1B ConOps SV-1 to Level 1C ICD 1:1 Parity Mandate**: Detailed internal wire-level interconnects, pinouts, RS-485 serial framing, opcodes (0x10, 0x11, etc.), register bitmasks, and CRC-16 equations belong in Level 1C Interface Control Documents (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`). Every ConOps SV-1 architectural edge/connection (`CONN-01`..`CONN-N`) MUST map 1:1 bidirectionally with a registered Level 1C interface entry in the System Interface Matrix (`ICD_01_SYSTEM_INTERFACE_MATRIX.md`), ensuring zero orphan connection links or unregistered interface contracts across Level 1B and Level 1C specifications. Section 8 (Op-Tx) captures high-level operational information exchanges and strictly excludes component-internal serial opcode reference tables.
- **Canonical SSOT Architecture Views Library Mandate (`docs/architecture/views/`) & Diagram Divergence Prevention**: All high-level operational and system architecture views MUST derive from or maintain 1:1 bidirectional parity with the canonical SSOT Architecture Views Library in `docs/architecture/views/` (including `sv1_super_system.mmd`, `sv1_platform_subsystems.mmd`, and `icd_system_interface_matrix.mmd`). Authoring independent, duplicate inline Mermaid architecture diagrams that diverge in subgraphs, nodes, ports, or connection links across ConOps, ICDs, and executive deliverables is strictly prohibited (refs #276).
- **Modularized Subgraph Boundary Edge Routing (Rule E5)**: In accordance with Rule E5, architecture diagrams MUST decouple super-system operational context (DoDAF SV-1 Super-System View `sv1_super_system.mmd`) from internal subsystem bus breakdowns (DoDAF SV-1 Platform Subsystems View `sv1_platform_subsystems.mmd`). Inter-segment and cross-subgraph communication MUST route through discrete boundary interface ports or segment gateway/controller nodes, strictly prohibiting edges penetrating deeply into nested subgraphs across multiple hierarchy levels. Internal component buses and tier interconnects route within dedicated subsystem views, eliminating monolithic visual hairballs (refs #277).

---

## The Six Core ConOps & Mission Intent Invariants

### 1. Pure Open Schema Contract ($N \ge N_{\mathrm{min}}$)
- **Zero Static Row Caps**: Artificial upper bounds, hardcoded array limits, or truncation heuristics in tabular specifications or lists are strictly forbidden. All schema contracts defined in `.pipeline/schemas/conops_specification_schema.json` and `.pipeline/schemas/mission_intent_specification_schema.json` are open collections ($N \ge N_{\mathrm{min}}$).
- **Minimum Cardinality Enforcement**: Specifications MUST satisfy domain-specific minimum cardinalities without restriction on upper expansion:
  * **Emergency Decision Matrix**: Minimum 7 canonical triggers ($N \ge 7$) covering `EMG-01` through `EMG-07` (Lost C2, Navigation Loss, Propulsion Failure, Sensor Fault, Geofence Breach, Structural Anomaly, Emergency / Critical Process Termination).
  * **PACE C2 Plan**: Minimum 4 communication tiers ($N \ge 4$) covering `Primary`, `Alternate`, `Contingency`, and `Emergency`.
  * **METL Tasks**: Minimum 1 task ($N \ge 1$), expanding to full doctrinal mission scope.
  * **Threat Matrix**: Minimum 1 threat ($N \ge 1$), expanding to all applicable operational threats.
  * **User Classes**: Minimum 1 class ($N \ge 1$).
  * **UAF Operational Activities**: Minimum 1 activity ($N \ge 1$).
  * **Operational Information Exchanges (Op-Tx)**: Minimum 1 exchange ($N \ge 1$).

### 2. Open Multi-Domain Operational Threat Matrices
- **9-Domain Minimum Requirement**: To guarantee comprehensive risk characterization beyond simple mechanical hardware failures, the Multi-Domain Operational Threat & Contested Environment Matrix MUST explicitly address at least one threat vector from every one of the following 9 operational threat domains ($N \ge 9$):
1. **Kinetic**: Ballistic impact, hostile fragments/shrapnel, physical obstacle/debris collision, explosive blast overpressure.
2. **Mechanical**: Structural resonance/flutter, mechanical fatigue failure, linkage or actuator mechanism jamming, bearing seizure, structural delamination or material fracture.
3. **Power / Thermal**: Energy storage / battery cell thermal runaway, power rail brownout, power converter / motor driver thermal throttling, generator disconnect.
4. **Environmental**: Extreme ambient temperature, dynamic fluid/aerodynamic/hydrodynamic loads exceeding structural design limits, probe/sensor freeze and icing, electrostatic/lightning discharge, heavy precipitation, atmospheric or fluid particulate contamination.
5. **EW / Cyber**: Positioning/navigation signal spoofing or jamming, RF command uplink jamming, telemetry interception, man-in-the-middle packet injection, unauthorized command injection, firmware tampering.
6. **Optical**: High-energy laser blinding of optical tracking sensors, sensor dazzling, camera saturation, optical flow denial.
7. **Signature / Acoustic**: Acoustic emission harmonics, infrared plume radiation, radar cross-section (RCS) observability.
8. **Human Factors**: Operator input disparity, operator cognitive saturation or fatigue, unauthorized control override, human-machine interface (HMI) communication protocol desynchronization.
9. **CBRN**: Chemical plumes, biological particulates, radiological contamination, toxic corrosive environments.

Restricting threat analysis to an arbitrary single domain or omitting applicable threat vectors is strictly prohibited.

### 3. Mandatory INCOSE SEH v5.0 MoE/MoP Mathematical Formulations
- **Mathematical Formulations**: Every Measure of Effectiveness (MoE) and Measure of Performance (MoP) defined in Section 3 of Mission Intent MUST provide an explicit mathematical equation or formula expressing the metric in KaTeX format.
- **Threshold & Objective Value Pairs**: Every metric MUST declare both a minimum acceptable **Threshold** performance value and an optimal **Objective** target value.
- **SI & Normalized Units**: Every metric MUST define its unit of measurement using normalized SI units, percentages, or explicit `Dimensionless` designations.

### 4. 100% Public Clause Citations
- **Traceability Rule**: Every operational requirement, normative standard, threat mitigation rule, and METL task MUST cite verifiable, publicly accessible standard clauses (e.g., `ISO/IEC/IEEE 29148:2018 §6.4.2`, general authoritative public standards clauses, `MIL-STD-882E §4.3`, `IEC 61508-3 §7.4`).
- **Prohibition of Un-Cited Additions**: Speculative, ungrounded, or heuristic requirements lacking authoritative clause citations are strictly forbidden across all specification units.

### 5. LaTeX & KaTeX Mathematical Rendering Integrity
Per [`rules/latex-katex-integrity.md`](latex-katex-integrity.md):
- **Display Math Blocks**: All mathematical formulations (derived from schema AST constraint or calculation definitions) MUST use dedicated display blocks `$$ \begin{aligned} ... \end{aligned} $$` on separate newlines.
- **Pure Symbolic Math**: Embedding physical unit macros (e.g. `\text{ m}`, `\text{ m/s}`, `\text{ J}`) inside LaTeX math blocks is strictly prohibited.
- **Accompanying Parameter Tables**: Display equations MUST be immediately followed by a parameter definition table defining every mathematical symbol, nominal value, SI unit, and engineering constraint.
- **No Table Math Delimiters**: Math delimiters (`$ ... $` or `$$ ... $$`) inside Markdown table headers, delimiter rows, and data cells are strictly prohibited. Use standard plain text and Unicode characters (e.g. `boundary_max m`, `v_max m/s`, `J`, `deg`, `tau_max ms`).

### 6. Deterministic Modular Assembly & Cross-Model Allocation
- **Modular Unit Storage**: ConOps and Mission Intent specifications MUST be authored as discrete modular unit files under `docs/conops/units/conops/` (12 modules: `01_METADATA_AND_OVERVIEW.md` through `12_EMERGENCY_DECISION_MATRIX.md`) and `docs/conops/units/mission_intent/` (10 modules: `01_COMMANDERS_INTENT.md` through `10_OPERATIONAL_ALLOCATION_TAGS.md`).
- **Deterministic Assembly Engine**: Master documents (`docs/conops/CONOPS.md` and `docs/conops/MISSION_INTENT.md`) MUST be compiled via `./target/release/assemble-conops`.
- **Zero Placeholder Tokens**: Assembled specifications MUST contain zero unresolved `{{...}}` template tokens.
- **Gate 24 Operational Allocation**: Every UAF Operational Activity (`OA-XX`) and METL Task (`MET-XX`) MUST define a machine-verifiable Gate 24 allocation tag (`/// OperationalAllocation: [OA-XX]` or `/// OperationalAllocation: [MET-XX]`) linking operational tasks to structural and behavioral SysML v2 AST elements.

---

## The 3-Tier Multi-Run Safety & Threat Derivation Lifecycle

Safety, threat, and hazard derivation in DEAP follows a strict three-tier, multi-run architectural lifecycle to eliminate circular dependency deadlocks between operational concept definition and physical component selection:

### Tier 1 (Run 1: Mission Intent / Concept Formulation)
- **Primary Methodologies**: Functional Hazard Assessment (FHA) under **IEC 61508-1 §7.4** and Operational Hazard Analysis (OHA) under **MIL-STD-882E Task 202**.
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

## Pure Schema-Driven Mathematical Formulation Invariant

All mathematical equations and parameter definition tables in ConOps must derive 100% and exclusively from AST nodes (`constraint def`, `calc def`, and attribute constraints) present in the user-provided SysML schemas in `schema/`.

Universal Formatting Contract: Every mathematical constraint rendered as a display block `$$ \begin{aligned} ... \end{aligned} $$` must be followed by a parameter definition table specifying symbols, values, units, and descriptions extracted directly from the schema AST node attributes.

The compiler contains ZERO pre-baked equations.

---

## Why

Treating operational concepts and mission intent as informal, ad-hoc prose leads to unverified operational risks, untraced mission requirements, under-dimensioned safety containment buffers, and catastrophic system failures during contingency events.

Enforcing strict schema contracts, open multi-domain threat modeling, mathematical formulations, public clause citations, and deterministic assembly ensures that Level 1B operational concepts are as rigorous and machine-verifiable as low-level source code.
