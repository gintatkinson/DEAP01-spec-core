| Attribute | Value |
| :--- | :--- |
| **Title** | Cited Research Inventory & Normative Baseline: {{SYSTEM_IDENTIFIER}} |
| **Version** | {{DOCUMENT_VERSION}} |
| **Date** | {{DOCUMENT_DATE}} |

# Cited Research Inventory & Normative Baseline: {{SYSTEM_IDENTIFIER}}

## 1. Scope & System Identification
- **System Identifier:** `{{SYSTEM_IDENTIFIER}}`
- **Operational Domain:** `{{OPERATIONAL_DOMAIN}}`
- **Research Scope:** {{RESEARCH_SCOPE_DESCRIPTION}}
- **Applicability Statement:** {{APPLICABILITY_STATEMENT}}

## 2. Applicable Regulatory & Domain Standards Baseline
| Standard ID | Issuing Body | Title / Baseline | Applicable Clauses | Verification Scope | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps, §6.4.3 OpsCon, §8.4 System Requirements | System & Operational Requirements | Adopted |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles & DLI/VCI Interfaces | System Architecture & Interoperability | Adopted |
| RTCA DO-178C / DO-254 | RTCA / EUROCAE | Software and Electronic Hardware Considerations in Airborne Systems | §6.3 Software Architecture, §11.0 Software Life Cycle Data | Software / Hardware Safety Assurance | Adopted |
| SAE ARP4754A / ARP4761 | SAE | Guidelines for Development of Civil Aircraft and Systems / Safety Assessment | §5.0 System Safety Assessment, FHA/PSSA/SSA | System Safety Process | Adopted |
| MIL-STD-882E | DoD | System Safety | Task 201 Preliminary Hazard Analysis, Task 205 System Hazard Analysis | System Hazard Analysis & Risk Mitigation | Adopted |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB), Annex C (Air Risk) | Operational Risk & Containment | Adopted |
| ASTM F3269-17 / F3411-22a | ASTM International | Standard Practice for Methods to Safely Bound Flight Behavior / Remote ID | Bounded Behavior & Remote Identification | Flight Control Bounds & Remote Identification | Adopted |
| RTCA DO-365B | RTCA | Minimum Operational Performance Standards for Detect and Avoid Systems | DAA Sensors, Algorithms, Alerting & Guidance | Detect and Avoid (DAA) Capabilities | Adopted |

## 3. Declared-Total Population Register
The Declared-Total Population Register catalogs every applicable normative obligation, safety constraint, METL task, and control pattern with its mandatory formal public clause citation. Un-cited additions are strictly prohibited.

| Obligation ID | Category | Formal Clause Citation | Obligation / Constraint Statement | Target Subsystem / Port | Verification Method |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Normative Obligation | {{OBL_01_CLAUSE_CITATION}} | {{OBL_01_STATEMENT}} | {{OBL_01_TARGET}} | {{OBL_01_VERIFICATION}} |
| `OBL-02` | Normative Obligation | {{OBL_02_CLAUSE_CITATION}} | {{OBL_02_STATEMENT}} | {{OBL_02_TARGET}} | {{OBL_02_VERIFICATION}} |
| `SAF-01` | Safety Constraint | {{SAF_01_CLAUSE_CITATION}} | {{SAF_01_STATEMENT}} | {{SAF_01_TARGET}} | {{SAF_01_VERIFICATION}} |
| `SAF-02` | Safety Constraint | {{SAF_02_CLAUSE_CITATION}} | {{SAF_02_STATEMENT}} | {{SAF_02_TARGET}} | {{SAF_02_VERIFICATION}} |
| `MET-01` | METL Task | {{MET_01_CLAUSE_CITATION}} | {{MET_01_STATEMENT}} | {{MET_01_TARGET}} | {{MET_01_VERIFICATION}} |
| `MET-02` | METL Task | {{MET_02_CLAUSE_CITATION}} | {{MET_02_STATEMENT}} | {{MET_02_TARGET}} | {{MET_02_VERIFICATION}} |
| `CTL-01` | Control Pattern | {{CTL_01_CLAUSE_CITATION}} | {{CTL_01_STATEMENT}} | {{CTL_01_TARGET}} | {{CTL_01_VERIFICATION}} |
| `CTL-02` | Control Pattern | {{CTL_02_CLAUSE_CITATION}} | {{CTL_02_STATEMENT}} | {{CTL_02_TARGET}} | {{CTL_02_VERIFICATION}} |

## 4. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | {{OBL_01_STANDARD_ID}} | {{OBL_01_CLAUSE_CITATION}} | {{OBL_01_CLAUSE_TITLE}} | Phase 1 (Structural) | `docs/features/{{OBL_01_FEATURE_SLUG}}.md` |
| `OBL-02` | {{OBL_02_STANDARD_ID}} | {{OBL_02_CLAUSE_CITATION}} | {{OBL_02_CLAUSE_TITLE}} | Phase 1.5 (Interfaces) | `docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md` |
| `SAF-01` | {{SAF_01_STANDARD_ID}} | {{SAF_01_CLAUSE_CITATION}} | {{SAF_01_CLAUSE_TITLE}} | Phase 2 (Behavioral) | `docs/user-stories/{{SAF_01_STORY_SLUG}}.md` |
| `SAF-02` | {{SAF_02_STANDARD_ID}} | {{SAF_02_CLAUSE_CITATION}} | {{SAF_02_CLAUSE_TITLE}} | Phase 2 (Behavioral) | `docs/safety/STPA_SYSTEM_THEORETIC_PROCESS_ANALYSIS.md` |
| `MET-01` | {{MET_01_STANDARD_ID}} | {{MET_01_CLAUSE_CITATION}} | {{MET_01_CLAUSE_TITLE}} | Phase 3 (Interaction) | `docs/use-cases/{{MET_01_USECASE_SLUG}}.md` |
| `MET-02` | {{MET_02_STANDARD_ID}} | {{MET_02_CLAUSE_CITATION}} | {{MET_02_CLAUSE_TITLE}} | Phase 3 (Interaction) | `docs/conops/MISSION_INTENT.md` |
| `CTL-01` | {{CTL_01_STANDARD_ID}} | {{CTL_01_CLAUSE_CITATION}} | {{CTL_01_CLAUSE_TITLE}} | Phase 1 / 2 | `docs/features/{{CTL_01_FEATURE_SLUG}}.md` |
| `CTL-02` | {{CTL_02_STANDARD_ID}} | {{CTL_02_CLAUSE_CITATION}} | {{CTL_02_CLAUSE_TITLE}} | Phase 1.5 (Interfaces) | `docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md` |

## 5. Normative Completeness & Gap Analysis
| Metric Parameter | Value | Target Threshold | Compliance Status |
| :--- | :--- | :--- | :--- |
| Declared Total Normative Obligations | {{TOTAL_OBLIGATIONS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total Safety Constraints | {{TOTAL_SAFETY_CONSTRAINTS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total METL Tasks | {{TOTAL_METL_TASKS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total Control Patterns | {{TOTAL_CONTROL_PATTERNS_COUNT}} | $\ge 1$ | Conforming |
| Clause Citation Traceability Percentage | {{CLAUSE_CITATION_PCT}}% | 100% | Conforming |
| Un-Cited / Speculative Additions | 0 | 0 (Strict Zero Tolerance) | Conforming |
