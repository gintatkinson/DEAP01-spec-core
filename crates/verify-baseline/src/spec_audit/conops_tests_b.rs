use super::*;

#[test]
fn test_issue_256_conops_schema_rejects_missing_super_system_and_subsystem_architecture() {
    let invalid_schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": [
            "metadata",
            "operational_context",
            "deficiencies",
            "proposed_capabilities",
            "user_classes"
        ],
        "properties": {
            "metadata": { "type": "object" },
            "operational_context": { "type": "object" }
        }
    });
    let findings = validate_conops_schema_contract(
        &invalid_schema,
        ".pipeline/schemas/conops_specification_schema.json",
    );
    assert!(
        !findings.is_empty(),
        "Schema contract must reject omission of super_system_architecture and subsystem_architecture"
    );
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "conops-schema-super-system-architecture-missing"));
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "conops-schema-subsystem-architecture-missing"));
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "conops-schema-interface-topology-missing"));
}

#[test]
fn test_issue_256_conops_doc_rejects_missing_super_system_and_subsystem_architecture() {
    let content = r#"# Concept of Operations (ConOps)
## 1. Operational Context
System boundaries and context.

## 2. Operational Deficiencies
Current gaps and issues.
"#;
    let super_findings = validate_super_system_architecture(content, "docs/conops/CONOPS.md");
    assert!(
        !super_findings.is_empty(),
        "Must reject ConOps doc missing super-system architecture"
    );
    assert_eq!(
        super_findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-super-system-architecture-missing")
    );

    let subsys_findings = validate_subsystem_architecture(content, "docs/conops/CONOPS.md");
    assert!(
        !subsys_findings.is_empty(),
        "Must reject ConOps doc missing subsystem architecture"
    );
    assert_eq!(
        subsys_findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-subsystem-architecture-missing")
    );

    let topo_findings = validate_system_interface_topology(content, "docs/conops/CONOPS.md");
    assert!(
        !topo_findings.is_empty(),
        "Must reject ConOps doc missing interface topology"
    );
    assert_eq!(
        topo_findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-system-interface-topology-missing")
    );
}

#[test]
fn test_issue_256_conops_schema_valid_contract_passes() {
    let valid_schema = serde_json::json!({
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
                    "properties": { "id": { "type": "string", "pattern": "^UCL-[0-9]{2}$" } }
                }
            }
        }
    });

    let findings = validate_conops_schema_contract(&valid_schema, ".pipeline/schemas/conops_specification_schema.json");
    assert!(findings.is_empty(), "Valid schema contract must pass with zero findings, got: {:?}", findings);
}

#[test]
fn test_issue_256_conops_doc_valid_architecture_passes() {
    let valid_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1, the operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)
- Auxiliary Support Segment (SEG-03)

```mermaid
flowchart TD
    subgraph External_Actors["External Operating Environment & Actors (IEEE 1362 §5.1)"]
        direction TB
        Operator["Operator"]
    end
    subgraph Supervisory_Segment["Supervisory Control Segment (IEEE 1362 §5.3)"]
        direction TB
        GCS["Ground Control Station<br/>• PORT-SUP-C2 (INOUT)"]
    end
    subgraph Core_Execution_Segment["Core System Execution Segment (DoDAF SV-1)"]
        direction TB
        Platform["Core Platform<br/>• PORT-CTRL-C2 (INOUT)"]
    end
    Operator -->|"CONN-01: Supervisory Command"| GCS
    GCS -->|"CONN-02: System Datalink"| Platform
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
    let super_f = validate_super_system_architecture(valid_conops, "docs/conops/CONOPS.md");
    assert!(
        super_f.is_empty(),
        "Valid super-system architecture must pass: {:?}",
        super_f
    );

    let subsys_f = validate_subsystem_architecture(valid_conops, "docs/conops/CONOPS.md");
    assert!(
        subsys_f.is_empty(),
        "Valid subsystem architecture must pass: {:?}",
        subsys_f
    );

    let topo_f = validate_system_interface_topology(valid_conops, "docs/conops/CONOPS.md");
    assert!(
        topo_f.is_empty(),
        "Valid interface topology must pass: {:?}",
        topo_f
    );
}

#[test]
fn test_issue_256_live_conops_schema_file_passes() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("resolve workspace root")?;
    let schema_path = root
        .join(".pipeline")
        .join("schemas")
        .join("conops_specification_schema.json");
    if schema_path.is_file() {
        let findings = validate_conops_schema_file(&schema_path)?;
        assert!(
            findings.is_empty(),
            "Live .pipeline/schemas/conops_specification_schema.json must pass contract validation: {:?}",
            findings
        );
    }
    Ok(())
}
