use super::*;

#[test]
fn test_issue_267_unconstrained_diagram_lacking_ieee_1362_and_dodaf_sv1_fails() {
    let unconstrained_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
Operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)

```mermaid
flowchart TD
    GCS["Ground Control Station"] --> Platform["Core Platform"]
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems derived deterministically from AST part definitions:
- Primary Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core flight guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
    let findings =
        validate_super_system_architecture(unconstrained_conops, "docs/conops/CONOPS.md");
    assert!(
        !findings.is_empty(),
        "Unconstrained architecture diagram lacking IEEE 1362 and DoDAF SV-1 must fail quality gate"
    );
    let codes: Vec<&str> = findings.iter().map(|f| f.rule_id.as_str()).collect();
    assert!(
        codes.contains(&"conops-super-system-standards-governance-missing"),
        "Expected finding 'conops-super-system-standards-governance-missing', got: {:?}",
        codes
    );
    assert!(
        codes.contains(&"conops-super-system-ports-missing"),
        "Expected finding 'conops-super-system-ports-missing', got: {:?}",
        codes
    );
    assert!(
        codes.contains(&"conops-super-system-connections-missing"),
        "Expected finding 'conops-super-system-connections-missing', got: {:?}",
        codes
    );
}

#[test]
fn test_issue_267_compliant_diagram_with_ieee_1362_and_dodaf_sv1_passes() {
    let compliant_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1, the operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)

```mermaid
flowchart TD
    subgraph External_Actors["External Operating Environment & Actors (IEEE 1362 §5.1)"]
        direction TB
        Operator["Operator"]
    end
    subgraph Supervisory_Segment["Supervisory Segment (IEEE 1362 §5.3)"]
        direction TB
        GCS["Ground Control Station<br/>• PORT-SUP-C2 (INOUT)"]
    end
    subgraph Core_Execution_Segment["Core System Execution Segment (DoDAF SV-1)"]
        direction TB
        Platform["Core Platform<br/>• PORT-CTRL-C2 (INOUT)"]
    end
    Operator -->|"CONN-01: Supervisory Command"| GCS
    GCS -->|"CONN-02: Datalink"| Platform
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems derived deterministically from AST part definitions:
- Primary Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core flight guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
    let findings =
        validate_super_system_architecture(compliant_conops, "docs/conops/CONOPS.md");
    assert!(
        findings.is_empty(),
        "Compliant architecture diagram with IEEE 1362 and DoDAF SV-1 must pass quality gate, got: {:?}",
        findings
    );
}

#[test]
fn test_issue_277_monolithic_nested_subgraph_cross_hierarchy_edges_detected_and_rejected() {
    let hairball_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1:

```mermaid
flowchart TD
    subgraph Super_System["Super-System (IEEE 1362)"]
        direction TB
        subgraph External_Actors["External Actors"]
            direction TB
            Operator["Operator<br/>• PORT_OP (OUT)"]
        end
        subgraph Supervisory_Segment["Supervisory Segment"]
            direction TB
            GCS["Ground Control Station<br/>• PORT_SUP (INOUT)"]
        end
        subgraph Platform_Segment["Platform Segment"]
            direction TB
            subgraph Tier_1["Tier 1 - Primary Flight"]
                direction TB
                Controller["Flight Computer<br/>• PORT_CTRL (INOUT)"]
            end
            subgraph Tier_2["Tier 2 - Actuation"]
                direction TB
                Actuator["Actuator Core<br/>• PORT_ACT (IN)"]
            end
        end
    end
    Operator -->|"CONN-01: Command"| GCS
    GCS -->|"CONN-02: Datalink"| Controller
    Controller -->|"CONN-03: Actuation Demand"| Actuator
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems:
- Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
    let findings = validate_super_system_architecture(hairball_conops, "docs/conops/CONOPS.md");
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "conops-super-system-modular-edge-routing-missing"),
        "Monolithic SV-1 diagram with 3-level nested subgraphs and cross-hierarchy edge routing into internal tiers must be rejected with 'conops-super-system-modular-edge-routing-missing' finding. Got findings: {:?}",
        findings
    );
}

#[test]
fn test_issue_277_modularized_boundary_edge_routing_passes() {
    let modular_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1:

```mermaid
flowchart TD
    subgraph Supervisory_Segment["Supervisory Segment"]
        direction TB
        GCS["Ground Control Station<br/>• PORT_SUP (INOUT)"]
    end
    subgraph Platform_Segment["Platform Segment"]
        direction TB
        Controller["Flight Computer<br/>• PORT_CTRL (INOUT)"]
        Actuator["Actuator Core<br/>• PORT_ACT (IN)"]
    end
    GCS -->|"CONN-01: Datalink"| Controller
    Controller -->|"CONN-02: Actuation Demand"| Actuator
```
"#;
    let findings = validate_super_system_architecture(modular_conops, "docs/conops/CONOPS.md");
    assert!(
        !findings
            .iter()
            .any(|f| f.rule_id == "conops-super-system-modular-edge-routing-missing"),
        "Modular SV-1 diagram with single-level segment subgraphs must pass Rule E5 validation. Got findings: {:?}",
        findings
    );
}

#[test]
fn test_issue_291_conops_schema_rejects_user_class_uc_pattern_collision() {
    let invalid_schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["metadata", "super_system_architecture", "subsystem_architecture", "system_interface_topology", "user_classes"],
        "properties": {
            "super_system_architecture": {
                "type": "object",
                "required": ["segments", "boundary_diagram_mermaid"],
                "properties": {
                    "segments": { "type": "array", "items": { "type": "object", "required": ["segment_id", "segment_name", "operational_role"] } },
                    "boundary_diagram_mermaid": { "type": "string" }
                }
            },
            "subsystem_architecture": {
                "type": "object",
                "required": ["subsystems", "ast_part_allocations"],
                "properties": {
                    "subsystems": { "type": "array", "items": { "type": "object", "required": ["subsystem_id", "subsystem_name", "ast_part_ref", "functional_description"] } },
                    "ast_part_allocations": { "type": "array", "items": { "type": "string" } }
                }
            },
            "system_interface_topology": {
                "type": "object",
                "required": ["interface_links"],
                "properties": {
                    "interface_links": { "type": "array", "items": { "type": "object", "required": ["link_id", "source_subsystem", "target_subsystem", "protocol_type"] } }
                }
            },
            "user_classes": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["id", "title", "player_or_operator", "interfacing_stakeholder", "characteristics", "training_type", "constraint_source"],
                    "properties": { "id": { "type": "string", "pattern": "^UC-[0-9]{2}$" } }
                }
            }
        }
    });
    let findings = validate_conops_schema_contract(&invalid_schema, ".pipeline/schemas/conops_specification_schema.json");
    assert!(
        findings.iter().any(|f| f.rule_id == "conops-schema-user-classes-taxonomy-collision"),
        "Schema with legacy ^UC-[0-9]{{2}}$ pattern must be rejected with 'conops-schema-user-classes-taxonomy-collision' (refs #291)"
    );
}

#[test]
fn test_issue_291_conops_doc_rejects_user_class_uc_collision() {
    let colliding_conops = r#"# Concept of Operations (ConOps)
## 4. User Classes & Stakeholder Taxonomy
| User Class ID | Title | Player or Operator | Interfacing Stakeholder | Characteristics & Responsibilities | Training & Qualification | Constraint Source |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UC-01** | System Operator (SO) | Direct Operator | Safety Authority | Supervision oversight | Type Qualification | ISO 29148 |
| **UC-04** | Maintenance Technician (MT) | Support Operator | Maintenance Depot | Pre-sortie inspections | Certified Tech | ISO 29148 |

Operational handover is executed between incoming operator and outgoing operator (UC-01).
"#;
    let findings = validate_user_classes(colliding_conops, "docs/conops/04_USER_CLASSES.md");
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "conops-user-class-taxonomy-collision" && f.message.contains("UC-01")),
        "ConOps declaring User Class UC-01 must be rejected with taxonomy collision finding (refs #291)"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "conops-user-class-taxonomy-collision" && f.message.contains("UC-04")),
        "ConOps declaring User Class UC-04 must be rejected with taxonomy collision finding (refs #291)"
    );
}

#[test]
fn test_issue_291_conops_doc_ucl_taxonomy_passes() {
    let compliant_conops = r#"# Concept of Operations (ConOps)
## 4. User Classes & Stakeholder Taxonomy
| User Class ID | Title | Player or Operator | Interfacing Stakeholder | Characteristics & Responsibilities | Training & Qualification | Constraint Source |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCL-01** | System Operator (SO) | Direct Operator | Safety Authority | Supervision oversight | Type Qualification | ISO 29148 |
| **UCL-04** | Maintenance Technician (MT) | Support Operator | Maintenance Depot | Pre-sortie inspections | Certified Tech | ISO 29148 |

Operational handover is executed between incoming operator and outgoing operator (UCL-01).
"#;
    let findings = validate_user_classes(compliant_conops, "docs/conops/04_USER_CLASSES.md");
    assert!(
        findings.is_empty(),
        "ConOps with unambiguous UCL-xx taxonomy must pass with zero findings, got: {:?}",
        findings
    );
}
