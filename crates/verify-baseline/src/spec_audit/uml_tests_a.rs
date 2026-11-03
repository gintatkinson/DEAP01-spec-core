//! Unit tests for `spec_audit::uml_validator` (diagram and source reference gates).
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::BTreeMap;

use deap_core::sysml_cst::{ActionDef, AttributeDef, OperationDef, PartDef, StateDef, SysmlModel};

use crate::spec_audit::diagram_ast::{
    ClassAttributeAst, ClassDefAst, ClassDiagramAst, ClassOperationAst, SequenceDiagramAst,
    SequenceMessageAst, SequenceParticipantAst, StateDefAst, StateDiagramAst, StateTransitionAst,
    Visibility,
};
use crate::spec_audit::markdown_ast::{HeadingNode, MarkdownDoc, MermaidBlock, ProseSegment};
use crate::spec_audit::uml_validator::{
    validate_class_diagram, validate_sequence_diagram, validate_source_references,
    validate_state_diagram, validate_use_case_diagram_compliance, SysmlSymbolIndex,
};

pub(crate) fn create_test_sysml_model() -> SysmlModel {
    let mut model = SysmlModel::default();
    let mut flight_controller = PartDef {
        name: "FlightController".to_string(),
        ..Default::default()
    };
    flight_controller.attributes.push(AttributeDef {
        name: "altitude".to_string(),
        type_name: "float".to_string(),
        ..Default::default()
    });
    flight_controller.operations.push(OperationDef {
        name: "execute_pipeline".to_string(),
        return_type: Some("void".to_string()),
        ..Default::default()
    });
    flight_controller.actions.push(ActionDef {
        name: "arm_motors".to_string(),
        ..Default::default()
    });

    let mut sensor_hub = PartDef {
        name: "SensorHub".to_string(),
        ..Default::default()
    };
    sensor_hub.operations.push(OperationDef {
        name: "read_sensors".to_string(),
        ..Default::default()
    });

    model.parts.push(flight_controller);
    model.parts.push(sensor_hub);

    let state_idle = StateDef {
        name: "Idle".to_string(),
        entry_action: Some("init_subsystems()".to_string()),
        ..Default::default()
    };
    let state_active = StateDef {
        name: "Active".to_string(),
        ..Default::default()
    };
    if let Some(first_part) = model.parts.first_mut() {
        first_part.states.push(state_idle);
        first_part.states.push(state_active);
    }
    model
}

#[test]
fn test_symbol_indexing_and_lookup() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    assert!(index.part_names.contains("FlightController"));
    assert!(index.part_names.contains("SensorHub"));
    assert!(index.attribute_names.contains("altitude"));
    assert!(index.action_names.contains("execute_pipeline"));
    assert!(index.action_names.contains("read_sensors"));
    assert!(index.action_names.contains("arm_motors"));
    assert!(index.state_names.contains("Idle"));
    assert!(index.state_names.contains("Active"));
}

#[test]
fn test_valid_sequence_diagram_passes() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let seq = SequenceDiagramAst {
        participants: vec![
            SequenceParticipantAst { name: "Pilot".to_string(), alias: None, is_actor: true, line: 1 },
            SequenceParticipantAst { name: "FlightController".to_string(), alias: None, is_actor: false, line: 2 },
            SequenceParticipantAst { name: "SensorHub".to_string(), alias: None, is_actor: false, line: 3 },
        ],
        messages: vec![
            SequenceMessageAst {
                from: "Pilot".to_string(),
                to: "FlightController".to_string(),
                arrow: "->>".to_string(),
                message: "execute_pipeline()".to_string(),
                operation_name: Some("execute_pipeline".to_string()),
                parameters: Vec::new(),
                is_reply: false,
                line: 4,
            },
            SequenceMessageAst {
                from: "FlightController".to_string(),
                to: "SensorHub".to_string(),
                arrow: "->>".to_string(),
                message: "read_sensors()".to_string(),
                operation_name: Some("read_sensors".to_string()),
                parameters: Vec::new(),
                is_reply: false,
                line: 5,
            },
        ],
        activations: Vec::new(),
    };
    assert!(validate_sequence_diagram(&seq, &index).is_ok());
}

#[test]
fn test_unknown_lifeline_fails_sequence_validation() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let seq = SequenceDiagramAst {
        participants: vec![SequenceParticipantAst {
            name: "UnregisteredEngine".to_string(),
            alias: None,
            is_actor: false,
            line: 10,
        }],
        messages: Vec::new(),
        activations: Vec::new(),
    };
    let res = validate_sequence_diagram(&seq, &index);
    assert!(res.is_err());
    let errors = res.err().unwrap_or_default();
    assert!(errors
        .first()
        .is_some_and(|e| e.to_string().contains("Lifeline 'UnregisteredEngine' in sequence diagram")));
}

#[test]
fn test_unknown_operation_fails_sequence_validation() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let seq = SequenceDiagramAst {
        participants: vec![
            SequenceParticipantAst { name: "Pilot".to_string(), alias: None, is_actor: true, line: 1 },
            SequenceParticipantAst { name: "FlightController".to_string(), alias: None, is_actor: false, line: 2 },
        ],
        messages: vec![SequenceMessageAst {
            from: "Pilot".to_string(),
            to: "FlightController".to_string(),
            arrow: "->>".to_string(),
            message: "unregistered_method()".to_string(),
            operation_name: Some("unregistered_method".to_string()),
            parameters: Vec::new(),
            is_reply: false,
            line: 3,
        }],
        activations: Vec::new(),
    };
    let res = validate_sequence_diagram(&seq, &index);
    assert!(res.is_err());
    let errors = res.err().unwrap_or_default();
    assert!(errors.first().is_some_and(|e| e.to_string().contains("unregistered_method")));
}

#[test]
fn test_class_diagram_validation_passes() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let class_diag = ClassDiagramAst {
        classes: vec![ClassDefAst {
            name: "FlightController".to_string(),
            annotations: Vec::new(),
            attributes: vec![ClassAttributeAst {
                visibility: Some(Visibility::Public),
                name: "altitude".to_string(),
                type_name: Some("float".to_string()),
                raw: "+float altitude".to_string(),
                line: 2,
            }],
            operations: vec![ClassOperationAst {
                visibility: Some(Visibility::Public),
                name: "execute_pipeline".to_string(),
                parameters: Vec::new(),
                return_type: Some("void".to_string()),
                raw: "+execute_pipeline() void".to_string(),
                line: 3,
            }],
            line: 1,
        }],
        relations: Vec::new(),
    };
    assert!(validate_class_diagram(&class_diag, &index).is_ok());
}

#[test]
fn test_unknown_class_fails_class_validation() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let class_diag = ClassDiagramAst {
        classes: vec![ClassDefAst {
            name: "PhantomPart".to_string(),
            annotations: Vec::new(),
            attributes: Vec::new(),
            operations: Vec::new(),
            line: 1,
        }],
        relations: Vec::new(),
    };
    let res = validate_class_diagram(&class_diag, &index);
    assert!(res.is_err());
    let errors = res.err().unwrap_or_default();
    assert!(errors
        .first()
        .is_some_and(|e| e.to_string().contains("Class 'PhantomPart' in class diagram")));
}

#[test]
fn test_vacuous_class_stub_fails_validation() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let vacuous_diag = ClassDiagramAst {
        classes: vec![ClassDefAst {
            name: "FlightController".to_string(),
            annotations: Vec::new(),
            attributes: Vec::new(),
            operations: Vec::new(),
            line: 1,
        }],
        relations: Vec::new(),
    };
    let res = validate_class_diagram(&vacuous_diag, &index);
    assert!(res.is_err());
    let errors = res.err().unwrap_or_default();
    assert!(errors.first().is_some_and(|e| e
        .to_string()
        .contains("is a vacuous stub omitting declared AST ports and attributes")));
}

#[test]
fn test_class_diagram_with_port_passes() {
    let mut model = create_test_sysml_model();
    if let Some(first_part) = model.parts.first_mut() {
        first_part.ports.push(deap_core::sysml_cst::PortDef {
            name: "telemetry_port".to_string(),
            type_name: "TelemetryStream".to_string(),
            direction: "out".to_string(),
            ..Default::default()
        });
    }
    let index = SysmlSymbolIndex::from_model(&model);
    let class_diag = ClassDiagramAst {
        classes: vec![ClassDefAst {
            name: "FlightController".to_string(),
            annotations: Vec::new(),
            attributes: Vec::new(),
            operations: vec![ClassOperationAst {
                visibility: Some(Visibility::Public),
                name: "telemetry_port".to_string(),
                parameters: vec!["out".to_string()],
                return_type: Some("TelemetryStream".to_string()),
                raw: "+TelemetryStream telemetry_port (out)".to_string(),
                line: 2,
            }],
            line: 1,
        }],
        relations: Vec::new(),
    };
    assert!(validate_class_diagram(&class_diag, &index).is_ok());
}

#[test]
fn test_state_diagram_validation_passes() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let state_diag = StateDiagramAst {
        states: vec![
            StateDefAst { id: "Idle".to_string(), label: None, is_composite: false, sub_states: Vec::new(), line: 1 },
            StateDefAst { id: "Active".to_string(), label: None, is_composite: false, sub_states: Vec::new(), line: 2 },
        ],
        transitions: vec![StateTransitionAst {
            source: "Idle".to_string(),
            target: "Active".to_string(),
            event: Some("arm".to_string()),
            guard: None,
            action: Some("arm_motors()".to_string()),
            raw_label: Some("arm / arm_motors()".to_string()),
            line: 3,
        }],
        notes: Vec::new(),
    };
    assert!(validate_state_diagram(&state_diag, &index).is_ok());
}

#[test]
fn test_unknown_state_fails_state_validation() {
    let model = create_test_sysml_model();
    let index = SysmlSymbolIndex::from_model(&model);
    let state_diag = StateDiagramAst {
        states: vec![StateDefAst {
            id: "Exploded".to_string(),
            label: None,
            is_composite: false,
            sub_states: Vec::new(),
            line: 1,
        }],
        transitions: Vec::new(),
        notes: Vec::new(),
    };
    let res = validate_state_diagram(&state_diag, &index);
    assert!(res.is_err());
    let errors = res.err().unwrap_or_default();
    assert!(errors
        .first()
        .is_some_and(|e| e.to_string().contains("State 'Exploded' in state diagram")));
}

#[test]
fn test_source_references_section_validation() {
    let valid_doc = MarkdownDoc {
        headings: vec![
            HeadingNode { level: 1, text: "Epic 01".to_string(), line: 1 },
            HeadingNode { level: 2, text: "## 6. Source References".to_string(), line: 10 },
        ],
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: vec![ProseSegment {
            text: "Derived from `schema/model.sysml` and IEEE 29148 clause 8.4.".to_string(),
            line: 12,
        }],
        mermaid_blocks: Vec::new(),
        unresolved_placeholders: Vec::new(),
    };
    assert!(validate_source_references(&valid_doc).is_ok());

    let missing_doc = MarkdownDoc {
        headings: vec![HeadingNode { level: 1, text: "Epic 01".to_string(), line: 1 }],
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: Vec::new(),
        mermaid_blocks: Vec::new(),
        unresolved_placeholders: Vec::new(),
    };
    let res = validate_source_references(&missing_doc);
    assert!(res.is_err());
    assert!(res
        .err()
        .is_some_and(|e| e.to_string().contains("missing a mandatory 'Source References' section")));
}

#[test]
fn test_use_case_diagram_compliance_gate() {
    let legacy_doc = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([("primary_actor".to_string(), "HumanEngineer".to_string())]),
        prose: vec![],
        mermaid_blocks: vec![MermaidBlock {
            raw: "flowchart TD\n    Actor[\"HumanEngineer\"] --> Target[\"SystemVisionEngine\"]\n"
                .to_string(),
            header: "flowchart TD".to_string(),
            start_line: 10,
            end_line: 12,
        }],
        unresolved_placeholders: vec![],
    };

    let errs = validate_use_case_diagram_compliance(&legacy_doc, "docs/use-cases/uc-01.md");
    assert!(errs.iter().any(|e| e.contains("lacks a system boundary subgraph")));
    assert!(errs.iter().any(|e| e.contains("Directed arrows ('-->') used to connect actor")));
    assert!(errs.iter().any(|e| e.contains("Non-stadium nodes")));
    assert!(errs.iter().any(|e| e.contains("Trivial 2-node flow detected")));
}
