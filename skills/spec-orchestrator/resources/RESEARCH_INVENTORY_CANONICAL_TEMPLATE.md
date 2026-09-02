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

## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps, §6.4.3 OpsCon, §8.4 System Requirements | Requirements Engineering | 3 | ISO/IEC/IEEE 29148:2018 §6.4.2, §6.4.3, §8.4 |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles & DLI/VCI Interfaces | Interoperability | 2 | STANAG 4586 Ed. 4 §3.2, §4.1 |
| RTCA DO-178C / DO-254 | RTCA / EUROCAE | Software and Electronic Hardware Considerations in Airborne Systems | §6.3 Software Architecture, §11.0 Software Life Cycle Data | Safety Assurance | 2 | DO-178C §6.3, DO-254 §11.0 |
| SAE ARP4754A / ARP4761 | SAE | Guidelines for Development of Civil Aircraft and Systems / Safety Assessment | §5.0 System Safety Assessment, FHA/PSSA/SSA | Safety Assessment | 2 | ARP4754A §5.0, ARP4761 App. L |
| MIL-STD-882E | DoD | System Safety | Task 201 Preliminary Hazard Analysis, Task 205 System Hazard Analysis | Hazard Analysis | 2 | MIL-STD-882E Task 201, Task 205 |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB), Annex C (Air Risk) | Risk & Containment | 2 | SORA v2.5 Annex B, Annex C |
| ASTM F3269-17 / F3411-22a | ASTM International | Standard Practice for Methods to Safely Bound Flight Behavior / Remote ID | Bounded Behavior & Remote Identification | Flight Bounds & Remote ID | 2 | ASTM F3269-17 §5.2, F3411-22a §6.1 |
| RTCA DO-365B | RTCA | Minimum Operational Performance Standards for Detect and Avoid Systems | DAA Sensors, Algorithms, Alerting & Guidance | Detect & Avoid | 2 | RTCA DO-365B §2.2.4, §2.2.5 |

## 3. Declared-Total Population Register
The Declared-Total Population Register catalogs every applicable normative obligation, safety constraint, METL task, and control pattern with its mandatory formal public clause citation. Un-cited additions are strictly prohibited.

| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Normative Obligation | ISO/IEC/IEEE 29148:2018 | {{OBL_01_TARGET_METRIC}} | Inspection & Traceability Audit | {{OBL_01_CLAUSE_CITATION}} |
| `OBL-02` | Normative Obligation | NATO STANAG 4586 | {{OBL_02_TARGET_METRIC}} | Interface Conformance Test | {{OBL_02_CLAUSE_CITATION}} |
| `SAF-01` | Safety Constraint | RTCA DO-178C / DO-254 | {{SAF_01_TARGET_METRIC}} | Formal Verification & Safety Analysis | {{SAF_01_CLAUSE_CITATION}} |
| `SAF-02` | Safety Constraint | SAE ARP4754A / ARP4761 | {{SAF_02_TARGET_METRIC}} | System Hazard Analysis (FHA/PSSA) | {{SAF_02_CLAUSE_CITATION}} |
| `MET-01` | METL Task | MIL-STD-882E | {{MET_01_TARGET_METRIC}} | Operational Demonstration & Review | {{MET_01_CLAUSE_CITATION}} |
| `MET-02` | METL Task | JARUS SORA v2.5 | {{MET_02_TARGET_METRIC}} | Ground Risk Buffer & Flight Test | {{MET_02_CLAUSE_CITATION}} |
| `CTL-01` | Control Pattern | ASTM F3269-17 / F3411-22a | {{CTL_01_TARGET_METRIC}} | Hardware-in-the-Loop & Flight Bounds | {{CTL_01_CLAUSE_CITATION}} |
| `CTL-02` | Control Pattern | RTCA DO-365B | {{CTL_02_TARGET_METRIC}} | Sensor Alerting & DAA Guidance Run | {{CTL_02_CLAUSE_CITATION}} |

## 4. External Additions & Domain Extensions Registry
All external additions, proprietary extensions, and domain-specific baselines MUST carry authoritative public clause citations. Un-cited additions are strictly prohibited.

| Extension ID | Category | Standard / Baseline ID | Declared Total | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EXT-01` | Domain Extension | {{EXT_01_STANDARD_ID}} | {{EXT_01_DECLARED_TOTAL}} | {{EXT_01_TARGET_METRIC}} | Automated Conformance Test | {{EXT_01_CLAUSE_CITATION}} |
| `EXT-02` | External Addition | {{EXT_02_STANDARD_ID}} | {{EXT_02_DECLARED_TOTAL}} | {{EXT_02_TARGET_METRIC}} | Protocol Traceability Audit | {{EXT_02_CLAUSE_CITATION}} |

## 5. Clause-Level Allocation & Traceability Matrix
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

## 6. Normative Completeness & Gap Analysis
| Metric Parameter | Value | Target Threshold | Compliance Status |
| :--- | :--- | :--- | :--- |
| Declared Total Normative Obligations | {{TOTAL_OBLIGATIONS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total Safety Constraints | {{TOTAL_SAFETY_CONSTRAINTS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total METL Tasks | {{TOTAL_METL_TASKS_COUNT}} | $\ge 1$ | Conforming |
| Declared Total Control Patterns | {{TOTAL_CONTROL_PATTERNS_COUNT}} | $\ge 1$ | Conforming |
| Clause Citation Traceability Percentage | {{CLAUSE_CITATION_PCT}}% | 100% | Conforming |
| Un-Cited / Speculative Additions | 0 | 0 (Strict Zero Tolerance) | Conforming |
