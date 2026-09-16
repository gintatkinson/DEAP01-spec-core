| Attribute | Value |
| :--- | :--- |
| **Title** | System Operational Architecture & Physical Subsystem Decomposition |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE 1362:1998 Clause 5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1, the operational system is partitioned across major physical segments and operational boundaries.

{{SUPER_SYSTEM_ARCHITECTURE}}

### 4.2 Super-System Segment Allocation Matrix
Physical subsystems, computational nodes, and functional elements are allocated across operational segments to enforce clear physical and organizational ownership boundaries.

{{SEGMENT_ALLOCATION_MATRIX}}

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Detailed architectural decomposition of all physical and logical subsystems derived deterministically from the abstract SysML v2 AST part definitions.

{{SUBSYSTEM_ARCHITECTURE_SECTION}}

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of operational and physical interface ports connecting subsystems, communication buses, and external interaction points.

{{PORT_TAXONOMY_SECTION}}

### 4.5 Level 0 SSOT Model Binding Statement
Formal declaration binding this operational architecture specification to the authoritative Single Source of Truth (SSOT) Model-Based Systems Engineering (MBSE) AST schema.

{{MODEL_BINDING_STATEMENT}}
